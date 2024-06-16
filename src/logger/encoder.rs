// This encoder is based on the default encoder from the log crate.
// Only difference is the flattening of 'kvs'.
use std::collections::BTreeMap;
use std::fmt::Arguments;
use std::fmt::Display;
use std::option::IntoIter;
use std::thread;

use chrono::format::DelayedFormat;
use chrono::format::Fixed;
use chrono::format::Item;
use chrono::DateTime;
use chrono::Local;
use log::kv;
use log::kv::Key;
use log::kv::Source;
use log::kv::Value;
use log::kv::VisitSource;
use log::Level;
use log::Record;
use log4rs::encode::Encode;
use log4rs::encode::Write;
use serde::ser::Serialize;
use serde::ser::Serializer;
use serde_with::skip_serializing_none;

#[derive(Debug)]
pub(crate) struct JsonEncoder;

impl JsonEncoder {
  pub(crate) fn new() -> JsonEncoder {
    JsonEncoder {}
  }
}

impl JsonEncoder {
  fn encode_internal(&self, time: DateTime<Local>, w: &mut dyn Write, record: &Record) -> anyhow::Result<()> {
    let thread = thread::current();
    let message = Message {
      time: time.format_with_items(Some(Item::Fixed(Fixed::RFC3339)).into_iter()),
      message: record.args(),
      level: record.level(),
      module_path: record.module_path(),
      file: record.file(),
      line: record.line(),
      target: record.target(),
      thread: thread.name(),
      kvs: &collect(record.key_values())?,
    };

    message.serialize(&mut serde_json::Serializer::new(&mut *w))?;

    Ok(w.write_all("\n".as_bytes())?)
  }
}

impl Encode for JsonEncoder {
  fn encode(&self, w: &mut dyn Write, record: &Record) -> anyhow::Result<()> {
    let time = Local::now();
    self.encode_internal(time, w, record)
  }
}

#[skip_serializing_none]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Message<'a> {
  #[serde(serialize_with = "ser_display")]
  time: DelayedFormat<IntoIter<Item<'a>>>,
  #[serde(serialize_with = "ser_display")]
  message: &'a Arguments<'a>,
  level: Level,
  module_path: Option<&'a str>,
  file: Option<&'a str>,
  line: Option<u32>,
  target: &'a str,
  thread: Option<&'a str>,
  #[serde(flatten)]
  kvs: &'a BTreeMap<Key<'a>, Value<'a>>,
}

struct Collect<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
  fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
    self.0.insert(key, value);
    Ok(())
  }
}

fn collect<S>(source: &S) -> Result<BTreeMap<Key<'_>, Value<'_>>, kv::Error>
where
  S: Source + ?Sized,
{
  let mut visitor = Collect(BTreeMap::new());
  source.visit(&mut visitor)?;
  Ok(visitor.0)
}

fn ser_display<T, S>(v: &T, s: S) -> Result<S::Ok, S::Error>
where
  T: Display,
  S: Serializer,
{
  s.collect_str(v)
}

#[cfg(test)]
mod test {

  use super::*;
  use log4rs::encode::writer::simple::SimpleWriter;

  struct TestData<'a> {
    time: DateTime<Local>,
    message: &'a str,
    level: Level,
    module_path: &'a str,
    file: &'a str,
    line: u32,
    target: &'a str,
    thread_name: &'a str,
  }

  fn create_test_data(thread: &thread::Thread) -> TestData<'_> {
    TestData {
      time: Local::now(),
      message: "my log message",
      level: Level::Info,
      module_path: "test",
      file: "encoder.rs",
      line: 11_u32,
      target: "atlas-autoscaler",
      thread_name: thread.name().unwrap(),
    }
  }

  struct Fixture<'a> {
    encoder: JsonEncoder,
    writer: SimpleWriter<&'a mut Vec<u8>>,
  }

  fn create_fixture(buffer: &mut Vec<u8>) -> Fixture<'_> {
    Fixture {
      encoder: JsonEncoder::new(),
      writer: SimpleWriter(buffer),
    }
  }

  #[test]
  fn message_without_kv_serialized_correctly() {
    let mut buffer = vec![];
    let Fixture { encoder, mut writer } = create_fixture(&mut buffer);
    let thread: thread::Thread = thread::current();

    let TestData {
      time,
      message,
      level,
      module_path,
      file,
      line,
      target,
      thread_name,
    } = create_test_data(&thread);

    encoder
      .encode_internal(
        time,
        &mut writer,
        &Record::builder()
          .level(level)
          .target(target)
          .module_path(Some(module_path))
          .file(Some(file))
          .line(Some(line))
          .args(format_args!("{message}"))
          .build(),
      )
      .unwrap();

    let expected = format!(
      "{{\"time\":\"{timestamp}\",\"message\":\"{message}\",\"level\":\"{level}\",\"modulePath\":\"{module_path}\",\
             \"file\":\"{file}\",\"line\":{line},\"target\":\"{target}\",\"thread\":\"{thread_name}\"}}",
      timestamp = time.to_rfc3339(),
    );

    assert_eq!(expected, String::from_utf8(buffer).unwrap().trim());
  }

  #[test]
  fn message_with_kv_serialized_correctly() {
    let mut buffer = vec![];
    let Fixture { encoder, mut writer } = create_fixture(&mut buffer);
    let thread: thread::Thread = thread::current();

    let TestData {
      time,
      message,
      level,
      module_path,
      file,
      line,
      target,
      thread_name,
    } = create_test_data(&thread);

    let key = "projectStats";
    let value = 42_i32;
    let kvs = vec![(key, value)];

    encoder
      .encode_internal(
        time,
        &mut writer,
        &Record::builder()
          .level(level)
          .target(target)
          .module_path(Some(module_path))
          .file(Some(file))
          .line(Some(line))
          .args(format_args!("{message}"))
          .key_values(&Some(kvs))
          .build(),
      )
      .unwrap();

    let expected = format!(
      "{{\"time\":\"{timestamp}\",\"message\":\"{message}\",\"level\":\"{level}\",\"modulePath\":\"{module_path}\",\
             \"file\":\"{file}\",\"line\":{line},\"target\":\"{target}\",\"thread\":\"{thread_name}\",\"{key}\":{value}}}",
      timestamp = time.to_rfc3339(),
    );

    assert_eq!(expected, String::from_utf8(buffer).unwrap().trim());
  }
}
