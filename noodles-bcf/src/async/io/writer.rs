mod header;

use noodles_bgzf as bgzf;
use noodles_vcf::{self as vcf, header::StringMaps};
use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use self::header::write_header;
use crate::Record;

/// An async BCF writer.
pub struct Writer<W> {
    inner: W,
    string_maps: StringMaps,
    buf: Vec<u8>,
}

impl<W> Writer<W>
where
    W: AsyncWrite + Unpin,
{
    /// Returns a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// use noodles_bcf as bcf;
    /// let writer = bcf::r#async::io::Writer::from(io::sink());
    /// let _inner = writer.get_ref();
    /// ```
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Returns a mutable reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// use noodles_bcf as bcf;
    /// let mut writer = bcf::r#async::io::Writer::from(io::sink());
    /// let _inner = writer.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// use noodles_bcf as bcf;
    /// let writer = bcf::r#async::io::Writer::from(io::sink());
    /// let _inner = writer.into_inner();
    /// ```
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Writes a VCF header.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use noodles_vcf as vcf;
    ///
    /// let mut writer = bcf::r#async::io::Writer::new(io::sink());
    ///
    /// let header = vcf::Header::default();
    /// writer.write_header(&header).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_header(&mut self, header: &vcf::Header) -> io::Result<()> {
        write_file_format(&mut self.inner).await?;

        self.string_maps = StringMaps::try_from(header)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        write_header(&mut self.inner, header).await
    }

    /// Writes a record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use noodles_bcf as bcf;
    /// use noodles_vcf::{self as vcf, header::StringMaps};
    ///
    /// let mut writer = bcf::r#async::io::Writer::new(io::sink());
    ///
    /// let mut header = vcf::Header::builder()
    ///     .add_contig("sq0", Default::default())
    ///     .build();
    /// *header.string_maps_mut() = StringMaps::try_from(&header)?;
    ///
    /// writer.write_header(&header).await?;
    ///
    /// let record = bcf::Record::default();
    /// writer.write_record(&header, &record).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_record(&mut self, header: &vcf::Header, record: &Record) -> io::Result<()> {
        self.write_variant_record(header, record).await
    }

    /// Writes a variant record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use noodles_core::Position;
    /// use noodles_vcf as vcf;
    ///
    /// let mut writer = bcf::r#async::io::Writer::new(io::sink());
    ///
    /// let header = vcf::Header::builder()
    ///     .add_contig("sq0", Default::default())
    ///     .build();
    ///
    /// writer.write_header(&header).await?;
    ///
    /// let record = vcf::variant::RecordBuf::builder()
    ///     .set_reference_sequence_name("sq0")
    ///     .set_position(Position::MIN)
    ///     .set_reference_bases("A")
    ///     .build();
    ///
    /// writer.write_variant_record(&header, &record).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_variant_record(
        &mut self,
        header: &vcf::Header,
        record: &dyn vcf::variant::Record,
    ) -> io::Result<()> {
        use crate::io::writer::write_record;

        self.buf.clear();
        write_record(&mut self.buf, header, &self.string_maps, record)?;
        self.inner.write_all(&self.buf).await
    }
}

impl<W> Writer<bgzf::AsyncWriter<W>>
where
    W: AsyncWrite + Unpin,
{
    /// Creates an async BCF writer.
    ///
    /// The given stream is wrapped in a BGZF encoder.
    pub fn new(inner: W) -> Self {
        Self::from(bgzf::AsyncWriter::new(inner))
    }
}

impl<W> From<W> for Writer<W> {
    fn from(inner: W) -> Self {
        Self {
            inner,
            string_maps: StringMaps::default(),
            buf: Vec::new(),
        }
    }
}

async fn write_file_format<W>(writer: &mut W) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    use crate::{
        io::writer::{MAJOR, MINOR},
        MAGIC_NUMBER,
    };

    writer.write_all(MAGIC_NUMBER).await?;
    writer.write_u8(MAJOR).await?;
    writer.write_u8(MINOR).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_file_format() -> io::Result<()> {
        let mut buf = Vec::new();
        write_file_format(&mut buf).await?;

        let expected = [
            b'B', b'C', b'F', // magic
            0x02, // major
            0x02, // minor
        ];

        assert_eq!(buf, expected);

        Ok(())
    }
}
