use create_rust_app::Connection;
use fang::serde::{Deserialize, Serialize};
use fang::typetag;
use fang::PgConnection;
use fang::Runnable;
use fang::{FangError, Queueable, Scheduled};

use crate::models::todos::{CreateTodo, Todo};

#[derive(Serialize, Deserialize)]
#[serde(crate = "fang::serde")]
pub struct DailyTodo {
    pub text: String,
}

#[typetag::serde]
impl Runnable for DailyTodo {
    fn run(&self, queue: &dyn Queueable) -> Result<(), FangError> {
        println!("Adding daily todo {}", self.text);
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
        "sync".to_string()
    }

    // If `uniq` is set to true and the task is already in the storage, it won't be inserted again
    // The existing record will be returned for for any insertions operation
    fn uniq(&self) -> bool {
        true
    }

    fn cron(&self) -> Option<Scheduled> {
        // runs this task every minute, just for demonstration purposes
        //               sec  min   hour   day of month   month   day of week   year
        let expression = "0 * * * * * *";
        Some(Scheduled::CronPattern(expression.to_string()))
    }
}
