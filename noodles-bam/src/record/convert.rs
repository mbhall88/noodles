use std::io;

use noodles_sam as sam;

use super::{Record, ReferenceSequenceId};

impl Record {
    /// Converts a SAM record to a BAM record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bam as bam;
    /// use noodles_sam::{self as sam, header::ReferenceSequences};
    ///
    /// let reference_sequences = ReferenceSequences::default();
    /// let sam_record = sam::Record::default();
    ///
    /// let record = bam::Record::try_from_sam_record(&reference_sequences, &sam_record)?;
    /// assert_eq!(record, bam::Record::default());
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn try_from_sam_record(
        reference_sequences: &sam::header::ReferenceSequences,
        sam_record: &sam::Record,
    ) -> io::Result<Self> {
        use crate::{reader::record::read_record, writer::record::write_sam_record};

        let mut buf = Vec::new();
        write_sam_record(&mut buf, reference_sequences, sam_record)?;

        let mut reader = &buf[..];
        let mut record = Self::default();
        read_record(&mut reader, &mut record)?;

        Ok(record)
    }

    /// Converts this record to a SAM record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bam as bam;
    /// use noodles_sam as sam;
    ///
    /// let reference_sequences = sam::header::ReferenceSequences::default();
    ///
    /// let record = bam::Record::default();
    /// let sam_record = record.try_into_sam_record(&reference_sequences)?;
    ///
    /// assert_eq!(sam_record, sam::Record::default());
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn try_into_sam_record(
        &self,
        reference_sequences: &sam::header::ReferenceSequences,
    ) -> io::Result<sam::Record> {
        let mut builder = sam::Record::builder();

        let raw_read_name = self
            .read_name()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
            .and_then(|c_read_name| {
                c_read_name
                    .to_str()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
            })?;

        if raw_read_name != "*" {
            let read_name = raw_read_name
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

            builder = builder.set_read_name(read_name);
        }

        builder = builder.set_flags(self.flags());

        if let Some(reference_sequence_name) =
            get_reference_sequence_name(reference_sequences, self.reference_sequence_id())?
        {
            builder = builder.set_reference_sequence_name(reference_sequence_name);
        }

        if let Some(position) = self.position() {
            builder = builder.set_position(position);
        }

        builder = builder
            .set_mapping_quality(self.mapping_quality())
            .set_cigar(self.cigar().try_into()?);

        if let Some(mate_reference_sequence_name) =
            get_reference_sequence_name(reference_sequences, self.mate_reference_sequence_id())?
        {
            builder = builder.set_mate_reference_sequence_name(mate_reference_sequence_name);
        }

        if let Some(mate_position) = self.mate_position() {
            builder = builder.set_mate_position(mate_position);
        }

        builder = builder.set_template_length(self.template_length());

        if !self.sequence().is_empty() {
            builder = builder.set_sequence(self.sequence().into());
        }

        if !self.quality_scores().is_empty() {
            let quality_scores = self
                .quality_scores()
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            builder = builder.set_quality_scores(quality_scores);
        }

        let data = self
            .data()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        builder = builder.set_data(data);

        builder
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

fn get_reference_sequence_name(
    reference_sequences: &sam::header::ReferenceSequences,
    reference_sequence_id: Option<ReferenceSequenceId>,
) -> io::Result<Option<sam::record::ReferenceSequenceName>> {
    reference_sequence_id
        .map(i32::from)
        .map(|id| {
            reference_sequences
                .get_index(id as usize)
                .and_then(|(_, rs)| rs.name().parse().ok())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "invalid reference sequence ID")
                })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_reference_sequences(
    ) -> Result<sam::header::ReferenceSequences, sam::header::reference_sequence::NewError> {
        use sam::header::ReferenceSequence;

        [("sq0", 5), ("sq1", 8), ("sq2", 13)]
            .into_iter()
            .map(|(name, len)| ReferenceSequence::new(name, len).map(|rs| (name.into(), rs)))
            .collect()
    }

    fn build_record() -> Record {
        use sam::record::{Flags, MappingQuality};

        use crate::record::{Cigar, Data, QualityScores, Sequence};

        Record {
            ref_id: 1,
            pos: 61061,
            mapq: MappingQuality::from(12),
            bin: 4684,
            flag: Flags::PAIRED | Flags::READ_1,
            next_ref_id: 1,
            next_pos: 61152,
            tlen: 166,
            read_name: b"r0\x00".to_vec(),
            cigar: Cigar::from(vec![0x00000040]),    // 4M
            seq: Sequence::new(vec![0x18, 0x42], 4), // ATGC
            qual: QualityScores::from(vec![0x1f, 0x1d, 0x1e, 0x20]), // @>?A
            data: Data::from(vec![
                0x4e, 0x4d, 0x43, 0x00, // NM:i:0
                0x50, 0x47, 0x5a, 0x53, 0x4e, 0x41, 0x50, 0x00, // PG:Z:SNAP
            ]),
        }
    }

    #[test]
    fn test_try_into_sam_record() -> Result<(), Box<dyn std::error::Error>> {
        use sam::record::{
            cigar::{op, Op},
            data::{
                field::{Tag, Value},
                Field,
            },
        };

        let bam_record = build_record();
        let reference_sequences = build_reference_sequences()?;
        let actual = bam_record.try_into_sam_record(&reference_sequences)?;

        let expected = sam::Record::builder()
            .set_read_name("r0".parse()?)
            .set_flags(sam::record::Flags::PAIRED | sam::record::Flags::READ_1)
            .set_reference_sequence_name("sq1".parse()?)
            .set_position(sam::record::Position::try_from(61062)?)
            .set_mapping_quality(sam::record::MappingQuality::from(12))
            .set_cigar(sam::record::Cigar::from(vec![Op::new(op::Kind::Match, 4)]))
            .set_mate_reference_sequence_name("sq1".parse()?)
            .set_mate_position(sam::record::Position::try_from(61153)?)
            .set_template_length(166)
            .set_sequence("ATGC".parse()?)
            .set_quality_scores("@>?A".parse()?)
            .set_data(sam::record::Data::try_from(vec![
                Field::new(Tag::EditDistance, Value::Int(0)),
                Field::new(Tag::Program, Value::String(String::from("SNAP"))),
            ])?)
            .build()?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
