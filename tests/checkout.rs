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
        Some(_gref) => {
            unreachable!("should never be hit, as we only work with commits atm")
            // println!("we've hit a reference branch with : {:#?}", gref.name());
            // repo.set_head(gref.name().wrap_err("expected to be some")?)
        }
        // this is a commit, not a reference
        None => {
            println!("we've hit a commit branch with commit : {:#?}", object.id());
            repo.set_head_detached(object.id())
        }
    }?;
    Ok(())
}
pub fn checkout(
    source_id: near_verify_rs::types::source_id::SourceId,
) -> eyre::Result<(tempfile::TempDir, std::path::PathBuf)> {
    let tempdir = tempfile::tempdir()?;

    let target_dir = tempdir.path().to_path_buf();

    let SourceKind::Git(GitReference::Rev(rev)) = source_id.kind();
    println!("rev: {}", rev);
    checkout_remote_repo(source_id.url().as_str(), &target_dir, rev)?;

    for entry in std::fs::read_dir(&target_dir)? {
        let entry = entry?;
        println!("entry: {:?}", entry);
    }
    Ok((tempdir, target_dir))
}

const CHECKOUT_TEST_SNAPSHOT: &str = "git+https://github.com/dj8yfo/verify_contracts_collection?rev=e3303f0cf8761b99f84f93c3a2d7046be6f4edb5";

#[test]
#[ignore]
fn test_checkout() -> eyre::Result<()> {
    let source_id = near_verify_rs::types::source_id::SourceId::from_url(CHECKOUT_TEST_SNAPSHOT)
        .expect("no error");

    let (_tempdir, _target_dir) = checkout(source_id)?;

    Ok(())
}
