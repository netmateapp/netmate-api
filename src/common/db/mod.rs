use std::marker::PhantomData;

type Tb<T> = Table<T>;

pub struct Table<T>(&'static str, PhantomData<T>);

impl<T> Table<T> {
    pub const fn name(&self) -> &'static str {
        self.0
    }
}

pub const fn tb<T>(name: &'static str) -> Tb<T> {
    Table(name, PhantomData)
}

type Col<T> = Column<T>;

pub struct Column<T>(&'static str, PhantomData<T>);

impl<T> Column<T> {
    pub const fn name(&self) -> &'static str {
        self.0
    }
}

pub const fn col<T>(name: &'static str) -> Col<T> {
    Column(name, PhantomData)
}

pub mod account_handles {
    use super::{col, tb, Col, Tb};
    pub struct T;
    pub const ACCOUNT_HANDLES: Tb<T> = tb("account_handles");
    pub const ACCOUNT_ID: Col<T> = col("account_id");
    pub const HANDLE_ID: Col<T> = col("handle_id");
    pub const HANDLE_NAME: Col<T> = col("handle_name");
}