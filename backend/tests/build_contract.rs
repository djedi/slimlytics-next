#[test]
fn backend_container_includes_the_openapi_build_input() {
    let dockerfile = include_str!("../../docker/backend.Dockerfile");
    assert!(
        dockerfile.contains("COPY docs/openapi.json ./docs/openapi.json"),
        "backend uses include_str! for docs/openapi.json, so the Docker build must copy it"
    );
}
