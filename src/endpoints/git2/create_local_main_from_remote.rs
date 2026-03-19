use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use git2::{build::CheckoutBuilder, BranchType, Repository, StatusOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::PathBuf;

/// *`POST /pull/main/<repo_path>`*
///
/// Typically mounted as **`/git/pull/main/<repo_path>`**
///
/// Switches to the `main` branch (or creates a local branch tracking `origin/main`)
/// and fetches the latest commits from the remote.
#[post("/main/init/<repo_path..>")]
pub async fn create_local_main_from_remote(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components = repo_path.components();

    if !check_path_components(&mut path_components.clone()) {
        return not_ok_bad_repo_json_response();
    }

    let repo_path_string = format!(
        "{}{}{}",
        state.repo_dir.lock().unwrap().clone(),
        os_slash_str(),
        repo_path.display()
    );

    let repo = match Repository::open(&repo_path_string) {
        Ok(r) => r,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not open repo: {}", e)),
            );
        }
    };

    let mut status_opts = StatusOptions::new();
    status_opts
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut status_opts)) {
        Ok(s) => s,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Status check failed: {}", e)),
            );
        }
    };

    let has_changes = statuses.iter().any(|entry| {
        let s = entry.status();
        s.is_wt_modified()
            || s.is_wt_new()
            || s.is_wt_deleted()
            || s.is_index_modified()
            || s.is_index_new()
            || s.is_index_deleted()
    });

    if has_changes {
        return not_ok_json_response(
            Status::Conflict,
            make_bad_json_data_response(
                "Uncommitted changes detected. Commit before creating local main.".to_string(),
            ),
        );
    }

    if let Ok(mut remote) = repo.find_remote("origin") {
        if let Err(e) = remote.fetch(&["refs/heads/main:refs/remotes/origin/main"], None, None) {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Failed to fetch origin/main: {}", e)),
            );
        }
    } else {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response("No remote 'origin' found".to_string()),
        );
    }

    let branch_name = "main";

    if repo.find_branch(branch_name, BranchType::Local).is_ok() {
        return not_ok_json_response(
            Status::Conflict,
            make_bad_json_data_response("Local main branch already exists.".to_string()),
        );
    }

    let obj = match repo.revparse_single("refs/remotes/origin/main") {
        Ok(o) => o,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Cannot find remote branch origin/main: {}",
                    e
                )),
            );
        }
    };

    if let Err(e) = repo.branch(branch_name, &obj.peel_to_commit().unwrap(), false) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Failed to create local main branch: {}", e)),
        );
    }

    let obj = match repo.revparse_single(&format!("refs/heads/{}", branch_name)) {
        Ok(o) => o,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Cannot resolve local main branch: {}", e)),
            );
        }
    };

    if let Err(e) = repo.checkout_tree(&obj, Some(CheckoutBuilder::new().safe())) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Checkout failed: {}", e)),
        );
    }

    if let Err(e) = repo.set_head(&format!("refs/heads/{}", branch_name)) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Set HEAD failed: {}", e)),
        );
    }

    ok_ok_json_response()
}
