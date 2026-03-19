use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use git2::{Repository, build::CheckoutBuilder, StatusOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};

/// *`POST /branch/<branch_ref>/<repo_path>`*
///
/// Typically mounted as **`/git/branch/<branch_ref>/<repo_path>`**
///
/// Changes the selected branch for a given repo. The branch must exist.

#[post("/branch/<branch_ref>/<repo_path..>")]
pub async fn set_branch(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    branch_ref: String,
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
                make_bad_json_data_response(format!("could not open repo: {}", e)),
            );
        }
    };

    // 🔴 0. PREVENT CHECKOUT IF DIRTY
    let mut status_opts = StatusOptions::new();
    status_opts
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut status_opts)) {
        Ok(s) => s,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("status check failed: {}", e)),
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
                "Uncommitted changes detected. Commit or stash before switching branches."
                    .to_string(),
            ),
        );
    }

    // Full ref name (local branch)
    let branch_full = format!("refs/heads/{}", branch_ref);

    // 1. Resolve branch
    let obj = match repo.revparse_single(&branch_full) {
        Ok(o) => o,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("cannot resolve branch: {}", e)),
            );
        }
    };

    // 2. SAFE checkout (no force)
    if let Err(e) = repo.checkout_tree(
        &obj,
        Some(
            CheckoutBuilder::new()
                .safe() 
        ),
    ) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("checkout failed: {}", e)),
        );
    }

    // 3. Move HEAD
    if let Err(e) = repo.set_head(&branch_full) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("set_head failed: {}", e)),
        );
    }

    ok_ok_json_response()
}
