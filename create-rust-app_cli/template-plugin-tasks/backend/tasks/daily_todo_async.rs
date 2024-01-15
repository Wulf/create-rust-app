use fang::async_trait;
use fang::asynk::async_queue::AsyncQueueable;
use fang::serde::{Deserialize, Serialize};
use fang::{typetag, AsyncRunnable, FangError, Scheduled};

use crate::models::todos::{CreateTodo, Todo};

#[derive(Serialize, Deserialize)]
#[serde(crate = "fang::serde")]
pub struct DailyTodoAsync {
    pub text: String,
}

#[typetag::serde]
#[async_trait]
impl AsyncRunnable for DailyTodoAsync {
    async fn run(&self, _queueable: &mut dyn AsyncQueueable) -> Result<(), FangError> {
        println!("(async) Adding daily todo {}", self.text);
        let db = create_rust_app::Database::new();

        let con = &mut db.get_connection().unwrap();

        Todo::create(
            con,
            &CreateTodo {
                text: self.text.clone(),
            },
        )
        .unwrap();

        Ok(())
    }

    // This will be useful if you want to filter tasks.
    // the default value is `common`
    fn task_type(&self) -> String {
        "async".to_string()
    }

    // If `uniq` is set to true and the task is already in the storage, it won't be inserted again
    // The existing record will be returned for for any insertions operation
    fn uniq(&self) -> bool {
        true
    }

    // This will be useful if you would like to schedule tasks.
    // Default value is None (the task is not scheduled, it's just executed as soon as it's inserted)
    fn cron(&self) -> Option<Scheduled> {
        // runs this task every minute, just for demonstration purposes
        //               sec  min   hour   day of month   month   day of week   year
        let expression = "0 * * * * * *";
        Some(Scheduled::CronPattern(expression.to_string()))
    }

    // the maximum number of retries. Set it to 0 to make it not retryable
    // the default value is 20
    fn max_retries(&self) -> i32 {
        20
    }

    // backoff mode for retries
    fn backoff(&self, attempt: u32) -> u32 {
        u32::pow(2, attempt)
    }
}
