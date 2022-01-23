use bodhi::{BodhiClient, ContentType, FedoraRelease, Update, UpdateStatus};

use crate::output::progress_bar;

/// This helper function queries updates in "testing" state for a specific release, and prints a
/// nice progress bar to indicate query progress.
pub async fn query_testing(bodhi: &BodhiClient, release: FedoraRelease) -> Result<Vec<Update>, String> {
    let testing = "Updates (testing)";

    let testing_progress = |p, ps| progress_bar(testing, p, ps);

    let releases = vec![release];
    let testing_query = bodhi::query::UpdateQuery::new()
        .releases(&releases)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Testing)
        .callback(testing_progress);

    let testing_updates = match bodhi.paginated_request(&testing_query).await {
        Ok(updates) => updates,
        Err(error) => {
            return Err(error.to_string());
        },
    };

    Ok(testing_updates)
}

/// This helper function queries updates in "obsolete" state for a specific release, and prints a
/// nice progress bar to indicate query progress.
pub async fn query_obsoleted(bodhi: &BodhiClient, release: FedoraRelease) -> Result<Vec<Update>, String> {
    let obsolete = "Updates (obsolete)";
    let obsolete_progress = |p, ps| progress_bar(obsolete, p, ps);

    let releases = vec![release];
    let obsolete_query = bodhi::query::UpdateQuery::new()
        .releases(&releases)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Obsolete)
        .callback(obsolete_progress);

    let obsolete_updates = match bodhi.paginated_request(&obsolete_query).await {
        Ok(updates) => updates,
        Err(error) => {
            return Err(error.to_string());
        },
    };

    Ok(obsolete_updates)
}

/// This helper function queries updates in "pending" state for a specific release, and prints a
/// nice progress bar to indicate query progress.
pub async fn query_pending(bodhi: &BodhiClient, release: FedoraRelease) -> Result<Vec<Update>, String> {
    let pending = "Updates (pending)";
    let pending_progress = |p, ps| progress_bar(pending, p, ps);

    let releases = vec![release];
    let pending_query = bodhi::query::UpdateQuery::new()
        .releases(&releases)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Pending)
        .callback(pending_progress);

    let pending_updates = match bodhi.paginated_request(&pending_query).await {
        Ok(updates) => updates,
        Err(error) => {
            return Err(error.to_string());
        },
    };

    Ok(pending_updates)
}

/// This helper function queries updates in "unpushed" state for a specific release, and prints a
/// nice progress bar to indicate query progress.
pub async fn query_unpushed(bodhi: &BodhiClient, release: FedoraRelease) -> Result<Vec<Update>, String> {
    let unpushed = "Updates (unpushed)";
    let unpushed_progress = |p, ps| progress_bar(unpushed, p, ps);

    let releases = vec![release];
    let unpushed_query = bodhi::query::UpdateQuery::new()
        .releases(&releases)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Unpushed)
        .callback(unpushed_progress);

    let unpushed_updates = match bodhi.paginated_request(&unpushed_query).await {
        Ok(updates) => updates,
        Err(error) => {
            return Err(error.to_string());
        },
    };

    Ok(unpushed_updates)
}
