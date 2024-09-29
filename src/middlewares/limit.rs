use std::fmt::{self, Display};

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};

use crate::helper::redis::namespace::Namespace;

pub enum TimeUnit {
    SECS,
    MINS,
    HOURS,
    DAYS,
}

impl TimeUnit {
    pub fn apply(self, time_window: u32) -> TimeWindow {
        match self {
            TimeUnit::SECS => TimeWindow::seconds(time_window),
            TimeUnit::MINS => TimeWindow::minutes(time_window),
            TimeUnit::HOURS => TimeWindow::hours(time_window),
            TimeUnit::DAYS => TimeWindow::days(time_window),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TimeWindow(u32);

impl TimeWindow {
    pub const fn seconds(seconds: u32) -> Self {
        Self(seconds)
    }

    pub const fn minutes(minutes: u32) -> Self {
        Self::seconds(minutes * 60)
    }

    pub const fn hours(hours: u32) -> Self {
        Self::minutes(hours * 60)
    }

    pub const fn days(days: u32) -> Self {
        Self::hours(days * 24)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl ToRedisArgs for TimeWindow {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Count(u32);

impl Count {
    pub const fn new(quota: u32) -> Self {
        Self(quota)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<i32> for Count {
    fn from(value: i32) -> Self {
        Self::new(value as u32)
    }
}

impl FromRedisValue for Count {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        u32::from_redis_value(v).map(Count)
    }
}

impl FromCqlVal<Option<CqlValue>> for Count {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i32::from_cql(cql_val).map(Count::from)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InculsiveLimit(Count);

impl InculsiveLimit {
    pub const fn new(limit: Count) -> Self {
        Self(limit)
    }

    pub fn value(&self) -> Count {
        self.0
    }
}

impl FromCqlVal<Option<CqlValue>> for InculsiveLimit {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Count::from_cql(cql_val).map(InculsiveLimit::new)
    }
}

#[derive(Debug)]
pub struct EndpointName(Namespace);

impl EndpointName {
    pub const fn new(namespace: Namespace) -> Self {
        Self(namespace)
    }
}

impl Display for EndpointName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}