use std::{
    env,
    fs,
    path::Path,
    process::Command,
};

fn main() {
    let spec_path = Path::new("spec/firecracker.yaml");
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let openapi_path = out_dir.join("firecracker-openapi3.json");

    // Convert Swagger 2.0 → OpenAPI 3.0 (cached)
    if !openapi_path.exists() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let abs_spec = Path::new(&manifest_dir).join(spec_path);
        let status = Command::new("npx")
            .arg("swagger2openapi")
            .arg("--patch")
            .arg("-o")
            .arg(&openapi_path)
            .arg(&abs_spec)
            .status()
            .expect("failed to run swagger2openapi — is Node.js installed?");
        assert!(status.success(), "swagger2openapi conversion failed");
    }

    // Patch the spec for progenitor compatibility.
    //
    // Progenitor's extract_responses groups the `default` response into both
    // success and error buckets, then asserts that each bucket has at most one
    // distinct response type. With Code(200) + Default (different schemas),
    // this assertion fails. Fix: remove `default` (progenitor falls back to
    // Error::UnexpectedResponse for unlisted status codes).
    let raw = fs::read_to_string(&openapi_path).unwrap();
    let mut spec: serde_json::Value = serde_json::from_str(&raw).unwrap();
    if let Some(paths) = spec.get_mut("paths").and_then(|p| p.as_object_mut()) {
        for (_path, methods) in paths.iter_mut() {
            if let Some(methods) = methods.as_object_mut() {
                for (_method, op) in methods.iter_mut() {
                    if let Some(responses) =
                        op.get_mut("responses").and_then(|r| r.as_object_mut())
                    {
                        responses.remove("default");
                    }
                }
            }
        }
    }

    // Generate Rust client code via progenitor
    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    let mut generator = progenitor::Generator::new(&settings);
    let spec: openapiv3::OpenAPI = serde_json::from_value(spec).unwrap();
    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    fs::write(out_dir.join("codegen.rs"), content).unwrap();
}
