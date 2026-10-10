#[test]
fn readme_dependency_version_matches_crate() {
    version_sync::assert_markdown_deps_updated!("README.md");
}
