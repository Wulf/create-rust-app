// use crate::schema::*;
// use crate::diesel::*;

// use diesel::QueryResult;
// use serde::{Serialize, Deserialize};
// use crate::models::{PaginationParams, ID, UTC};
// use crate::DB;

// #[tsync::tsync]
// #[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Identifiable, Associations, AsChangeset)]
// #[table_name = "attachments"]
// pub struct Attachment {
//   pub id: ID,
  
//   pub name: String,
//   pub record_type: String,
//   pub record_id: ID,
//   pub blob_id: ID,

//   pub created_at: UTC,
// }

// #[belongs_to(Attachment)]
// pub struct AttachmentBlob {
//   pub id: ID,
  
//   key character varying COLLATE pg_catalog."default" NOT NULL,
//   filename character varying COLLATE pg_catalog."default" NOT NULL,
//   content_type character varying COLLATE pg_catalog."default",
//   metadata text COLLATE pg_catalog."default",
//   byte_size bigint NOT NULL,
//   pub checksum: String character varying COLLATE pg_catalog."default" NOT NULL,
  
//   pub created_at: UTC,
// }

// impl Todo {
//   pub fn create(db: &DB, item: &TodoChangeset) -> QueryResult<Self> {
//     use crate::schema::todos::dsl::*;
    
//     insert_into(todos)
//       .values(item)
//       .get_result::<Todo>(db)
//   }
  
//   pub fn read(db: &DB, item_id: ID) -> QueryResult<Self> {
//     use crate::schema::todos::dsl::*;
    
//     todos.filter(id.eq(item_id)).first::<Todo>(db)
//   }

//   pub fn read_all(db: &DB, pagination: &PaginationParams) -> QueryResult<Vec<Self>> {
//     use crate::schema::todos::dsl::*;

//     todos
//         .order(created_at)
//         .limit(pagination.page_size)
//         .offset(pagination.page * std::cmp::max(pagination.page_size, PaginationParams::MAX_PAGE_SIZE as i64))
//         .load::<Todo>(db)
//   }

//   pub fn update(db: &DB, item_id: ID, item: &TodoChangeset) -> QueryResult<Self> {
//     use crate::schema::todos::dsl::*;

//     diesel::update(todos.filter(id.eq(item_id)))
//         .set(item)
//         .get_result(db)
//   }

//   pub fn delete(db: &DB, item_id: ID) -> QueryResult<usize> {
//     use crate::schema::todos::dsl::*;

//     diesel::delete(todos.filter(id.eq(item_id))).execute(db)
//   }
// }