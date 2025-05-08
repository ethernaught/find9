use core::ptr;
use core::ffi::CStr;
use std::collections::HashMap;
use std::ffi::CString;
use std::io;
use crate::database::sql_value::SqlValue;

const SQLITE_OPEN_READWRITE: i32 = 0x00000002;
const SQLITE_OPEN_CREATE: i32 = 0x00000004;

#[derive(Clone)]
pub struct Database {
    db: usize//*mut u32
}

impl Database {

    pub fn open_existing(name: &str) -> io::Result<Self> {
        Self::open_with_flags(name, SQLITE_OPEN_READWRITE)
    }

    pub fn open_or_create(name: &str) -> io::Result<Self> {
        Self::open_with_flags(name, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE)
    }

    fn open_with_flags(name: &str, flags: i32) -> io::Result<Self> {
        let c_db_name = CString::new(name)?;
        let mut db: *mut u32 = ptr::null_mut();

        let rc = unsafe { sqlite3_open_v2(c_db_name.as_ptr(), &mut db, flags, ptr::null()) };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            db: db as usize
        })
    }

    pub fn create_table(&self, name: &str, columns: &HashMap<String, String>) {
        let mut column_definitions: Vec<String> = Vec::new();

        for (column_name, column_type) in columns {
            column_definitions.push(format!("{} {}", column_name, column_type));
        }

        let create_table = format!("CREATE TABLE IF NOT EXISTS {} (
            {}
        );", name, column_definitions.join(", "));

        unsafe { sqlite3_exec(self.db as *mut u32, CString::new(&*create_table).unwrap().as_ptr(), None, ptr::null_mut(), ptr::null_mut()) };
    }

    pub fn insert(&self, table: &str, fields: &HashMap<&str, SqlValue>) {
        let field_names: Vec<&str> = fields.keys().cloned().collect();
        let field_values: Vec<String> = fields.values().map(|v| match v {
            SqlValue::Int(i) => i.to_string(),
            SqlValue::Uint(i) => i.to_string(),
            SqlValue::Float(i) => i.to_string(),
            SqlValue::Str(s) => format!("'{}'", s.replace('\'', "''")),
        }).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({});",
            table,
            field_names.join(", "),
            field_values.join(", ")
        );

        unsafe { sqlite3_exec(self.db as *mut u32, CString::new(&*sql).unwrap().as_ptr(), None, ptr::null_mut(), ptr::null_mut()) };
    }

    pub fn get(&self, table: &str, fields: Option<Vec<&str>>, condition: Option<&str>) -> Vec<HashMap<String, String>> {
        let field_names = match fields {
            Some(f) => f.join(", "),
            None => "*".to_string()
        };

        let sql = match condition {
            Some(cond) => format!("SELECT {} FROM {} WHERE {}; ", field_names, table, cond),
            None => format!("SELECT {} FROM {}; ", field_names, table),
        };

        let mut documents = Vec::new();
        let query_cstr = CString::new(CString::new(sql).unwrap()).unwrap();

        unsafe {
            sqlite3_exec(
                self.db as *mut u32,
                query_cstr.as_ptr(),
                Some(query_callback),
                &mut documents as *mut Vec<HashMap<String, String>> as *mut u32,
                ptr::null_mut()
            );
        }

        documents
    }

    pub fn close(&self) {
        unsafe { sqlite3_close(self.db as *mut u32) };
    }
}
/*
fn execute_sql(db: *mut u32, sql: &str) -> i32 {
    let c_sql = CString::new(sql).unwrap();
    unsafe { sqlite3_exec(db, c_sql.as_ptr(), None, ptr::null_mut(), ptr::null_mut()) }
}
*/
#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_open_v2(filename: *const i8, db: *mut *mut u32, flags: i32, z_vfs: *const i8) -> i32;

    fn sqlite3_exec(
        db: *mut u32,
        sql: *const i8,
        callback: Option<extern "C" fn(*mut u32, i32, *mut *mut i8, *mut *mut i8) -> i32>,
        arg: *mut u32,
        errmsg: *mut *mut i8
    ) -> i32;

    fn sqlite3_close(db: *mut u32) -> i32;
}

extern "C" fn query_callback(_arg: *mut u32, column_count: i32, column_values: *mut *mut i8, column_names: *mut *mut i8) -> i32 {
    let documents: &mut Vec<HashMap<String, String>> = unsafe {
        &mut *( _arg as *mut Vec<HashMap<String, String>> )
    };

    let mut document = HashMap::new();

    for i in 0..column_count {
        let column_name = unsafe { CStr::from_ptr(*column_names.offset(i as isize)) };
        let value = unsafe { CStr::from_ptr(*column_values.offset(i as isize)) };

        document.insert(column_name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned());
    }

    documents.push(document);

    0
}
