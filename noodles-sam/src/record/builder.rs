use crate::{Cigar, Data, Flags, MappingQuality};

use super::{QualityScores, Record, Sequence, NULL_FIELD};

#[derive(Debug, Default)]
pub struct Builder {
    name: Option<String>,
    flags: Flags,
    reference_sequence_name: Option<String>,
    position: u32,
    mapping_quality: MappingQuality,
    cigar: Cigar,
    mate_reference_sequence_name: Option<String>,
    mate_position: u32,
    template_len: i32,
    sequence: Sequence,
    quality_scores: QualityScores,
    data: Data,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn set_flags(mut self, flags: Flags) -> Self {
        self.flags = flags;
        self
    }

    pub fn set_reference_sequence_name(mut self, reference_sequence_name: &str) -> Self {
        self.reference_sequence_name = Some(reference_sequence_name.into());
        self
    }

    pub fn set_position(mut self, position: u32) -> Self {
        self.position = position;
        self
    }

    pub fn set_mapping_quality(mut self, mapping_quality: MappingQuality) -> Self {
        self.mapping_quality = mapping_quality;
        self
    }

    pub fn set_cigar(mut self, cigar: Cigar) -> Self {
        self.cigar = cigar;
        self
    }

    pub fn set_mate_reference_sequence_name(mut self, mate_reference_sequence_name: &str) -> Self {
        self.mate_reference_sequence_name = Some(mate_reference_sequence_name.into());
        self
    }

    pub fn set_mate_position(mut self, mate_position: u32) -> Self {
        self.mate_position = mate_position;
        self
    }

    pub fn set_template_len(mut self, template_len: i32) -> Self {
        self.template_len = template_len;
        self
    }

    pub fn set_sequence(mut self, sequence: Sequence) -> Self {
        self.sequence = sequence;
        self
    }

    pub fn set_quality_scores(mut self, quality_scores: QualityScores) -> Self {
        self.quality_scores = quality_scores;
        self
    }

    pub fn set_data(mut self, data: Data) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Record {
        let null_field = || NULL_FIELD.into();

        Record {
            qname: self.name.unwrap_or_else(null_field),
            flag: self.flags,
            rname: self.reference_sequence_name.unwrap_or_else(null_field),
            pos: self.position,
            mapq: self.mapping_quality,
            cigar: self.cigar,
            rnext: self.mate_reference_sequence_name.unwrap_or_else(null_field),
            pnext: self.mate_position,
            tlen: self.template_len,
            seq: self.sequence,
            qual: self.quality_scores,
            data: self.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cigar, data};

    use super::*;

    #[test]
    fn test_default() {
        let record = Builder::new().build();

        assert_eq!(record.name(), "*");
        assert!(record.flags().is_empty());
        assert_eq!(record.reference_sequence_name(), "*");
        assert_eq!(record.position(), 0);
        assert_eq!(u8::from(record.mapping_quality()), 255);
        assert!(record.cigar().ops().is_empty());
        assert_eq!(record.mate_reference_sequence_name(), "*");
        assert_eq!(record.mate_position(), 0);
        assert_eq!(record.template_len(), 0);
        assert!(record.sequence().is_empty());
        assert!(record.quality_scores().is_empty());
        assert!(record.data().fields().is_empty());
    }

    #[test]
    fn test_build() -> Result<(), Box<dyn std::error::Error>> {
        let cigar = Cigar::new(vec![cigar::Op::new(cigar::op::Kind::Match, 4)]);

        let data = Data::new(vec![data::Field::new(
            String::from("NH"),
            data::Value::Int32(1),
        )]);

        let sequence: Sequence = "ATCGATC".parse()?;
        let quality_scores: QualityScores = "NOODLES".parse()?;

        let record = Builder::new()
            .set_name("r0")
            .set_flags(Flags::from(65))
            .set_reference_sequence_name("sq0")
            .set_position(13)
            .set_mapping_quality(MappingQuality::from(37))
            .set_cigar(cigar)
            .set_mate_reference_sequence_name("sq1")
            .set_mate_position(17)
            .set_template_len(4)
            .set_sequence(sequence.clone())
            .set_quality_scores(quality_scores.clone())
            .set_data(data)
            .build();

        assert_eq!(record.name(), "r0");
        assert_eq!(u16::from(record.flags()), 65);
        assert_eq!(record.reference_sequence_name(), "sq0");
        assert_eq!(record.position(), 13);
        assert_eq!(u8::from(record.mapping_quality()), 37);
        assert_eq!(record.cigar().ops().len(), 1);
        assert_eq!(record.mate_reference_sequence_name(), "sq1");
        assert_eq!(record.mate_position(), 17);
        assert_eq!(record.template_len(), 4);
        assert_eq!(record.sequence(), &sequence);
        assert_eq!(record.quality_scores(), &quality_scores);
        assert_eq!(record.data().fields().len(), 1);

        Ok(())
    }
}
