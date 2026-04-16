use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::{FetcherError, Result};

const CSV_HEADERS: [&str; 5] = [
    "comment_time",
    "comment_content",
    "user_id",
    "user_name",
    "profile_url",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentUserRecord {
    pub comment_time: String,
    pub comment_content: String,
    pub user_id: String,
    pub user_name: String,
    pub profile_url: String,
}

pub struct CommentUserCsvExporter {
    file_path: PathBuf,
    writer: Option<BufWriter<File>>,
}

impl CommentUserCsvExporter {
    pub fn new(project_root: &Path, live_id: &str) -> Self {
        let file_path = project_root
            .join("exports")
            .join(format!("chat_users_{live_id}.csv"));

        Self {
            file_path,
            writer: None,
        }
    }

    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// # Errors
    ///
    /// Returns an error if the CSV file cannot be read.
    pub fn read_comments(file_path: &Path) -> Result<Vec<CommentUserRecord>> {
        let file = File::open(file_path).map_err(FetcherError::Io)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut records = Vec::new();

        for result in rdr.records() {
            let record = result.map_err(|e| FetcherError::Internal(format!("CSV error: {e}")))?;
            if record.len() < 5 {
                continue;
            }
            records.push(CommentUserRecord {
                comment_time: record[0].to_string(),
                comment_content: record[1].to_string(),
                user_id: record[2].to_string(),
                user_name: record[3].to_string(),
                profile_url: record[4].to_string(),
            });
        }
        Ok(records)
    }

    pub fn append_comment(&mut self, record: &CommentUserRecord) -> Result<()> {
        let writer = self.ensure_writer()?;
        write_csv_row(
            writer,
            &[
                &record.comment_time,
                &record.comment_content,
                &record.user_id,
                &record.user_name,
                &record.profile_url,
            ],
        )?;
        writer.flush().map_err(FetcherError::Io)
    }

    fn ensure_writer(&mut self) -> Result<&mut BufWriter<File>> {
        if self.writer.is_none() {
            let parent = self.file_path.parent().ok_or_else(|| {
                FetcherError::Internal(
                    "Comment user CSV path is missing parent directory".to_string(),
                )
            })?;
            fs::create_dir_all(parent).map_err(FetcherError::Io)?;

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path)
                .map_err(FetcherError::Io)?;
            let should_write_header = file.metadata().map_err(FetcherError::Io)?.len() == 0;

            let mut writer = BufWriter::new(file);
            if should_write_header {
                write_csv_row(&mut writer, &CSV_HEADERS)?;
                writer.flush().map_err(FetcherError::Io)?;
            }
            self.writer = Some(writer);
        }

        self.writer.as_mut().ok_or_else(|| {
            FetcherError::Internal("Failed to initialize comment user CSV writer".to_string())
        })
    }
}

fn write_csv_row<W: Write>(writer: &mut W, fields: &[&str]) -> Result<()> {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            writer.write_all(b",").map_err(FetcherError::Io)?;
        }
        write_csv_field(writer, field)?;
    }
    writer.write_all(b"\n").map_err(FetcherError::Io)?;
    Ok(())
}

fn write_csv_field<W: Write>(writer: &mut W, value: &str) -> Result<()> {
    if value.contains([',', '"', '\n', '\r']) {
        writer.write_all(b"\"").map_err(FetcherError::Io)?;
        for ch in value.chars() {
            if ch == '"' {
                writer.write_all(b"\"\"").map_err(FetcherError::Io)?;
            } else {
                write!(writer, "{ch}").map_err(FetcherError::Io)?;
            }
        }
        writer.write_all(b"\"").map_err(FetcherError::Io)?;
        return Ok(());
    }

    writer
        .write_all(value.as_bytes())
        .map_err(FetcherError::Io)?;
    Ok(())
}
