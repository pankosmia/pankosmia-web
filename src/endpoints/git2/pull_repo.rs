use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, not_ok_offline_json_response,
    ok_json_response,
};
use git2::{AutotagOption, FetchOptions, RemoteUpdateFlags, Repository};
use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use serde_json::json;
use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;
use std::time::SystemTime;

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    // println!("{}", msg);
    lb.set_target(rc.id(), &msg).expect("Set FF target failed");
    repo.set_head(&name).expect("set_head failed");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("checkout head failed");
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<bool, git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;
    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(true);
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(false)
}

/// *`POST /pull-repo/<remote_name>/<repo_path>`*
///
/// Typically mounted as **`/git/pull-repo/<remote_name>/<repo_path>`**
///
/// Pulls (fetches and merges) for a repo.
#[post("/pull-repo/<remote_name>/<repo_path..>")]
pub async fn pull_repo(
    state: &State<AppSettings>,
    remote_name: &str,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string().clone()
        );
        let remote_transport_regex = Regex::new(r"^(https?|ssh)://|git@").unwrap();
        if remote_transport_regex.is_match(&remote_name) && !NET_IS_ENABLED.load(Ordering::Relaxed) {
            return not_ok_offline_json_response();
        }
        match Repository::open(&repo_path_string) {
            Ok(repo) => {
                let mut remote = repo
                    .find_remote(remote_name)
                    .or_else(|_| repo.remote_anonymous(remote_name))
                    .expect("could not find remote");
                let mut fo = FetchOptions::new();
                match remote.download(&[] as &[&str], Some(&mut fo)) {
                    Ok(_) => {}
                    Err(e) => {
                        return not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(
                                format!("could not fetch repo: {}", e).to_string(),
                            ),
                        )
                    }
                };
                remote.disconnect().expect("could not disconnect remote");
                remote
                    .update_tips(
                        None,
                        RemoteUpdateFlags::UPDATE_FETCHHEAD,
                        AutotagOption::Unspecified,
                        None,
                    )
                    .expect("could not update tips");
                let fetch_head_ref = repo
                    .find_reference("FETCH_HEAD")
                    .expect("Could not find reference FETCH_HEAD");
                let fetch_commit = repo
                    .reference_to_annotated_commit(&fetch_head_ref)
                    .expect("Could not find fetch commit");
                let analysis = repo
                    .merge_analysis(&[&fetch_commit])
                    .expect("Could not do analysis");
                let mut merge_type = "fast-forward";
                let mut has_conflicts = false;
                if analysis.0.is_fast_forward() {
                    let head = repo.head().expect("Could not locate head");
                    let head_branch_name =
                        head.name().expect("Could not get branch name from head");
                    match repo.find_reference(&format!("{}", &head_branch_name)) {
                        Ok(mut r) => {
                            fast_forward(&repo, &mut r, &fetch_commit)
                                .expect("Could not fast forward");
                        }
                        Err(e) => {
                            return not_ok_json_response(
                                Status::InternalServerError,
                                make_bad_json_data_response(
                                    format!(
                                        "could not find branch reference {}: {}",
                                        head_branch_name, e
                                    )
                                    .to_string(),
                                ),
                            )
                        }
                    };
                } else if analysis.0.is_normal() {
                    merge_type = "normal";
                    let head_commit = repo
                        .reference_to_annotated_commit(
                            &repo.head().expect("Could not get repo head"),
                        )
                        .expect("could not get reference to head");
                    has_conflicts = normal_merge(&repo, &head_commit, &fetch_commit)
                        .expect("Could not normal merge");
                } else {
                    merge_type = "up-to-date";
                }
                let metadata_path =
                    format!("{}{}metadata.json", &repo_path_string, os_slash_str(),);
                std::fs::File::open(&metadata_path)
                    .expect("Could not open metadata file")
                    .set_modified(SystemTime::now())
                    .expect("Could not set timestamp of metadata file");
                let response_json = json!({"is_good":true, "reason":"ok", "merge_type": merge_type, "has_conflicts": has_conflicts});
                ok_json_response(serde_json::to_string(&response_json).unwrap())
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
