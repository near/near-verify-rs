use eyre::ContextCompat;
use near_verify_rs::types::source_id::{GitReference, SourceKind};

fn checkout_remote_repo(
    repo_url: &str,
    target_path: &std::path::Path,
    rev_str: &str,
) -> eyre::Result<()> {
    let repo = git2::Repository::clone_recurse(repo_url, target_path)?;

    let oid = git2::Oid::from_str(rev_str)?;
    let _commit = repo.find_commit(oid)?;

    let (object, reference) = repo.revparse_ext(rev_str)?;

    repo.checkout_tree(&object, None)?;

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => {
            println!("we've hit a reference branch with : {:#?}", gref.name());
            repo.set_head(gref.name().wrap_err("expected to be some")?)
        }
        // this is a commit, not a reference
        None => {
            println!("we've hit a commit branch with commit : {:#?}", object.id());
            repo.set_head_detached(object.id())
        }
    }?;
    Ok(())
}

#[test]
fn test_checkout() -> eyre::Result<()> {
    let source_id = near_verify_rs::types::source_id::SourceId::from_url(
        SIMPLE_PACKAGE_SNAPSHOT_GIT_STANDALONE,
    )
    .expect("no error");

    let target_dir = std::path::PathBuf::from("./i_want_to_check_test_checkout_here".to_string());

    let SourceKind::Git(GitReference::Rev(rev)) = source_id.kind();
    println!("rev: {}", rev);
    checkout_remote_repo(source_id.url().as_str(), &target_dir, rev)?;
    Ok(())
}

const SIMPLE_PACKAGE_SNAPSHOT_GIT_STANDALONE: &str = "git+https://github.com/dj8yfo/verify_contracts_collection?rev=e3303f0cf8761b99f84f93c3a2d7046be6f4edb5";

const SIMPLE_PACKAGE_META: &str = r#"
{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=e3303f0cf8761b99f84f93c3a2d7046be6f4edb5"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/e3303f0cf8761b99f84f93c3a2d7046be6f4edb5",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#;
