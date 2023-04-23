#![allow(unused_imports)]
#![allow(dead_code)]

mod constraint;
mod utils;

use crate::constraint::{Constraint, Dep, Ident, Onset, SonSeqPr, Syllabify};
use rand::{rngs::StdRng, Rng, SeedableRng};
use unicode_segmentation::UnicodeSegmentation;

use utils::{permute_delete, VecRet};

// string -> syllabified candidate -> random deletions (all winners generated via deletions) ->
// eval against constraints

const VOWELS: [&str; 7] = ["o", "ɛ", "ɔ", "i", "u", "a", "e"];

#[derive(Debug, Clone)]
pub struct SyllabifiedCandidate {
    form: Vec<Segment>,
    rng: StdRng,
}

#[derive(Debug, Clone)]
struct Segment {
    char: String,
    syllable_index: SyllableIndex,
    seg_type: SegmentType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SyllableIndex {
    Onset,
    Nucleus,
    Coda,
    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SegmentType {
    Vowel,
    Consonant,
}

impl SyllabifiedCandidate {
    fn delete(mut self) -> Self {
        if !self.form.is_empty() {
            self.form.remove(self.rng.gen_range(0..self.form.len()));
        }

        SyllabifiedCandidate {
            form: syllabify(self.clone().clear_indices().form),
            rng: self.rng,
        }
    }

    fn permute(&self) -> Vec<Self> {
        permute_delete(&self.form)
            .iter()
            .map(|form| SyllabifiedCandidate {
                form: form.to_owned(),
                rng: StdRng::seed_from_u64(7777777),
            })
            .collect()
    }

    fn clear_indices(self) -> Self {
        SyllabifiedCandidate {
            form: self
                .form
                .iter()
                .map(|seg| Segment {
                    char: seg.char.clone(),
                    syllable_index: SyllableIndex::None,
                    seg_type: seg.seg_type.clone(),
                })
                .collect(),
            rng: self.rng,
        }
    }
}

fn get_seg_type(grapheme: &str) -> SegmentType {
    if VOWELS.contains(&grapheme) {
        SegmentType::Vowel
    } else {
        SegmentType::Consonant
    }
}

impl From<&str> for SyllabifiedCandidate {
    fn from(str: &str) -> SyllabifiedCandidate {
        let mut graphemes: Vec<Segment> = str
            .graphemes(true)
            .map(|grapheme| Segment {
                char: grapheme.to_owned(),
                syllable_index: SyllableIndex::None,
                seg_type: get_seg_type(grapheme),
            })
            .collect();

        // grouping affricates together as non-separate segments
        // can we do this more cleanly?
        if !graphemes.is_empty() {
            for idx in 0..graphemes.len() - 1 {
                if graphemes[idx].char == "d\u{361}" && graphemes[idx + 1].char == "ʒ" {
                    let char_to_concat = graphemes[idx + 1].char.clone();
                    graphemes[idx].char.push_str(&char_to_concat);
                    graphemes.remove(idx + 1);
                }
            }
        }

        SyllabifiedCandidate {
            form: syllabify(graphemes),
            rng: StdRng::seed_from_u64(7777777),
        }
    }
}

impl From<SyllabifiedCandidate> for String {
    fn from(cand: SyllabifiedCandidate) -> String {
        cand.form.iter().map(|seg| seg.char.to_owned()).collect()
    }
}

impl Segment {
    fn morph_type(&self, seg_type: SyllableIndex) -> Segment {
        Segment {
            char: self.char.clone(),
            syllable_index: seg_type,
            seg_type: self.seg_type.clone(),
        }
    }
}

fn mark_vowels(candidate: Vec<Segment>) -> Vec<Segment> {
    // mark vowels
    // then extend onsets before vowels
    // then mark codas in remaining spaces after vowels
    // all segments should now be marked as onset, vowel, or coda

    candidate
        .iter()
        .map(|segment| {
            if VOWELS.contains(&segment.char.as_str()) {
                segment.morph_type(SyllableIndex::Nucleus)
            } else {
                segment.clone()
            }
        })
        .collect()
}

fn mark_onsets(candidate: Vec<Segment>) -> Vec<Segment> {
    let mut annotated: (SyllableIndex, Vec<Segment>) = candidate.iter().rev().fold(
        (SyllableIndex::None, Vec::new()),
        |(prev_seg_type, mut segs), seg| match (prev_seg_type, seg.to_owned().syllable_index) {
            (SyllableIndex::Nucleus, SyllableIndex::None) => (
                SyllableIndex::Onset,
                segs.push_ret(seg.morph_type(SyllableIndex::Onset))
                    .to_owned(),
            ),
            _ => (
                seg.syllable_index.to_owned(),
                segs.push_ret(seg.to_owned()).to_owned(),
            ),
        },
    );

    annotated.1.reverse();

    annotated.1
}

fn mark_codas(candidate: Vec<Segment>) -> Vec<Segment> {
    candidate
        .iter()
        .fold(
            (SyllableIndex::None, Vec::new()),
            |(prev_seg_type, mut segs), seg| match (prev_seg_type, seg.syllable_index.to_owned()) {
                (SyllableIndex::Nucleus, SyllableIndex::None) => (
                    SyllableIndex::Coda,
                    segs.push_ret(seg.morph_type(SyllableIndex::Coda))
                        .to_owned(),
                ),
                _ => (
                    seg.syllable_index.to_owned(),
                    segs.push_ret(seg.to_owned()).to_owned(),
                ),
            },
        )
        .1
}

fn syllabify(candidate: Vec<Segment>) -> Vec<Segment> {
    mark_codas(mark_onsets(mark_vowels(candidate)))
}

fn main() {
    let cand: SyllabifiedCandidate = "owókíowó".into();
    dbg!(cand.permute());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_delete_1() {
        let cand: SyllabifiedCandidate = "test".into();

        let cand_str: String = cand.delete().into();

        assert_eq!(cand_str, "tst");
    }

    #[test]
    fn test_delete_2() {
        let cand: SyllabifiedCandidate = "test".into();

        let cand_str: String = cand.delete().delete().delete().delete().delete().into();

        assert_eq!(cand_str, "");
    }

    #[test]
    fn test_delete_3() {
        let cand: SyllabifiedCandidate = "owókíowó".into();

        let cand_str: String = cand.delete().delete().delete().delete().into();

        assert_eq!(cand_str, "wówó");
    }

    #[test]
    fn test_delete_4() {
        let cand: SyllabifiedCandidate = "owókíowó".into();

        let cand_str: String = cand
            .delete()
            .delete()
            .delete()
            .delete()
            .delete()
            .delete()
            .delete()
            .delete()
            .into();

        assert_eq!(cand_str, "");
    }

    #[test]
    fn test_ident_1() {
        let cand1: SyllabifiedCandidate = "owókíowó".into();
        let cand2: SyllabifiedCandidate = "ówakíówó".into();

        let ident = Ident(cand1);

        assert_eq!(ident.evaluate(cand2), 3);
    }

    #[test]
    fn test_dep_1() {
        let syllabified_candidate: SyllabifiedCandidate = dbg!("owoktwiowo".into());
        assert_eq!(
            Dep(syllabified_candidate.clone())
                .evaluate(syllabified_candidate.delete().delete().delete().delete()),
            0
        )
    }

    #[test]
    fn test_ssp_1() {
        let syllabified_candidate: SyllabifiedCandidate = "owókíowó".into();
        assert_eq!(SonSeqPr.evaluate(syllabified_candidate), 7);
    }

    #[test]
    fn test_ssp_2() {
        let syllabified_candidate: SyllabifiedCandidate = "".into();
        assert_eq!(SonSeqPr.evaluate(syllabified_candidate), 0);
    }

    #[test]
    fn test_onset_1() {
        let syllabified_candidate: SyllabifiedCandidate = dbg!("owoktwiowo".into());
        assert_eq!(Onset.evaluate(syllabified_candidate), 2);
    }

    #[test]
    fn test_syllabify_constraint() {
        let syllabified_candidate: SyllabifiedCandidate = dbg!("owoktwiowo".into());
        assert_eq!(Syllabify.evaluate(syllabified_candidate), 1);
    }
}
