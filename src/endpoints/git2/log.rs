use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_json_response,
};
use git2::{ObjectType, Repository, Time};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use rocket::serde::Serialize;

fn print_time(time: &Time) -> String {
    let offset = time.offset_minutes();
    let (hours, minutes) = (offset / 60, offset % 60);
    let dt = time::OffsetDateTime::from_unix_timestamp(time.seconds()).unwrap();
    let dto = dt.to_offset(time::UtcOffset::from_hms(hours as i8, minutes as i8, 0).unwrap());
    let format = time::format_description::parse("[weekday repr:short] [month repr:short] [day padding:space] [hour]:[minute]:[second] [year] [offset_hour sign:mandatory][offset_minute]")
        .unwrap();
    dto.format(&format).unwrap()
}

#[derive(Serialize)]
struct CommitJson {
    id: String,
    author: String,
    date: String,
    message: String
}

/// *`GET /log/<repo_path>`*
///
/// Typically mounted as **`/git/log/<repo_path>`**
///
/// Returns the log of the repo.
#[get("/log/<repo_path..>")]
pub async fn log_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                let mut revwalk = repo.revwalk().expect("Could not revwalk repository");
                revwalk.set_sorting(git2::Sort::TIME).unwrap();
                let head = repo.head().expect("Could not locate head");
                let head_branch_name = head.name().expect("Could not get branch name from head");
                let rev_spec = repo.revparse(head_branch_name).expect("Could not rev_parse");
                if rev_spec.mode().contains(git2::RevparseMode::SINGLE) {
                    revwalk
                        .push(rev_spec.from().unwrap().id())
                        .expect("Could not push");
                } else {
                    let from = rev_spec.from().unwrap().id();
                    let to = rev_spec.to().unwrap().id();
                    revwalk.push(to).expect("could not push");
                    if rev_spec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                        let base = repo.merge_base(from, to).expect("could not merge_base");
                        let o = repo
                            .find_object(base, Some(ObjectType::Commit))
                            .expect("Could not find_object");
                        revwalk.push(o.id()).expect("could not push");
                    }
                    revwalk.hide(from).expect("could not hide");
                }
                revwalk.push_head().expect("could not push_head");
                let mut return_json = vec!();
                for rev_step in revwalk {
                    let commit_id = rev_step.expect("Could not unwrap rev_step");
                    let commit = repo.find_commit(commit_id).expect("Could not find commit");
                    return_json.push(CommitJson {
                        id: commit.id().to_string(),
                        author: commit.author().to_string(),
                        date: print_time(&commit.time()),
                        message: commit.message().unwrap_or("No Message").to_string(),
                    });
                }
                ok_json_response(serde_json::to_string(&return_json).unwrap())
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not open repo: {}", e)),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
