use common::db_pool;
use common::module::commit::Commit;
use common::module::error::AppError;
use common::module::share::Share;
use rbatis::{Page, PageRequest};
use uuid::Uuid;

pub struct CommitService {}

impl CommitService {
    pub async fn get_commit_page(
        share_id: &Uuid,
        pickup_code: Option<String>,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<Commit>, AppError> {
        Self::verify_code(share_id, pickup_code).await?;
        let page = Commit::select_page_by_shareid(
            db_pool!(),
            &PageRequest::new(page_no, page_size),
            share_id,
        )
        .await?;
        Ok(page)
    }

    pub async fn create_commit(
        user_id: &Uuid,
        share_id: &Uuid,
        pickup_code: Option<String>,
        context: String,
    ) -> Result<(), AppError> {
        Self::verify_code(share_id, pickup_code).await?;
        let commit = Commit::new(share_id.clone(), user_id.clone(), context);
        Commit::insert(db_pool!(), &commit).await?;
        Ok(())
    }

    pub async fn delete_commit(commit_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        Commit::delete_by_id(db_pool!(), commit_id, user_id).await?;
        Ok(())
    }

    pub async fn verify_code(share_id: &Uuid, pickup_code: Option<String>) -> Result<(), AppError> {
        let share = Share::select_by_id(db_pool!(), share_id)
            .await
            .map_err(|_e| AppError::ShareFileNotFound)?
            .get(0)
            .ok_or(AppError::ShareFileNotFound)?
            .to_owned();
        if !share.is_public.ok_or(AppError::ShareFileNotFound)? && share.pickup_code != pickup_code
        {
            return Err(AppError::ShareCodeMismatched);
        }
        Ok(())
    }
}
