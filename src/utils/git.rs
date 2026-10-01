use crate::structs::PankosmiaError;
use git2::{Repository, RepositoryInitOptions};

pub(crate) fn init_repo(
    repo_dir: &String,
    branch_name: Option<String>
) -> Result<Repository, PankosmiaError> {
        // Init repo
    let final_new_branch_name = branch_name.clone().unwrap_or("main".to_string());
    let mut repo_options = RepositoryInitOptions::new();
    let repo_options2 = repo_options.initial_head(final_new_branch_name.as_str());
    let new_repo = match Repository::init_opts(&repo_dir, &repo_options2) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(PankosmiaError(format!("Could not create repo: {}", e)));
        }
    };
    // Set up local user info
    let mut config = new_repo.config().expect("new repo config");
    config
        .set_str("user.name", whoami::username().as_str())
        .expect("config user name");
    config
        .set_str(
            "user.email",
            format!("{}@localhost", whoami::username().as_str()).as_str(),
        )
        .expect("config user email");
    Ok(new_repo)
}

pub(crate) fn add_and_commit_repo(repo: Repository, message: &String) -> Result<(), PankosmiaError> {
    // Add and commit
    repo
        .index()
        .expect("repo index")
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .expect("add all");
    repo.index().expect("repo index 2").write().expect("repo write");
    let sig = repo.signature().expect("repo sig");
    let tree_id = {
        let mut index = repo.index().expect("repo index 3");
        index.write_tree().expect("write tree")
    };
    let tree = repo.find_tree(tree_id).expect("find tree");
    repo
        .commit(Some("HEAD"), &sig, &sig, message.as_str(), &tree, &[])
        .expect("repo commit");
    Ok(())
}
