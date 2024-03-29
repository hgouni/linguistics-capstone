use crate::{SyllabifiedCandidate, SyllableIndex, UnderlyingIndex};
use similar::{DiffOp, TextDiff};

// need to make this a subtrait of debug since we need to tell rust that everything that implements
// Constraint must implement Debug since we're using trait objects
pub trait Constraint: std::fmt::Debug {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize;
}

#[derive(Debug)]
pub struct RankedConstraint {
    pub rank: usize,
    pub constraint: Box<dyn Constraint>,
}

impl Constraint for RankedConstraint {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        self.constraint.evaluate(surface)
    }
}

impl Constraint for Vec<&RankedConstraint> {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        self.iter()
            .fold(0, |prev, next| prev + next.evaluate(surface.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident(pub SyllabifiedCandidate);

impl Constraint for Ident {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let self_str: String = self.clone().0.into();
        let surface_str: String = surface.into();

        let diff = TextDiff::from_graphemes::<String>(&self_str, &surface_str);

        diff.ops()
            .iter()
            .filter(|op| matches!(op, DiffOp::Replace { .. }))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dep(pub SyllabifiedCandidate);

impl Constraint for Dep {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let self_str: String = self.clone().0.into();
        let surface_str: String = surface.into();

        let diff = TextDiff::from_graphemes::<String>(&self_str, &surface_str);

        diff.ops()
            .iter()
            .filter(|op| matches!(op, DiffOp::Insert { .. }))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Onset;

impl Constraint for Onset {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let syllabi = surface
            .form
            .iter()
            .filter(|seg| seg.syllable_index == SyllableIndex::Nucleus)
            .count();

        let onsets = surface
            .form
            .iter()
            .filter(|seg| seg.syllable_index == SyllableIndex::Onset)
            .count();

        (syllabi - onsets) * 3
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SonSeqPr;

impl Constraint for SonSeqPr {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        surface
            .form
            .iter()
            // a hack to ignore accent marks
            //
            // .next().unwrap() should never panic here bc that's only possible if the initial
            // candidate input string is empty, and if that's true, then the iterator will be empty
            .map(|seg| match seg.char.chars().next().unwrap() {
                'e' | 'ɛ' | 'o' | 'ɔ' => 1,
                'u' => 2,
                'i' => 3,
                _ => 0,
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Syllabify;

impl Constraint for Syllabify {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        surface
            .form
            .iter()
            .filter(|seg| seg.syllable_index == SyllableIndex::None)
            .count()
    }
}

#[derive(Debug)]
pub struct Max(pub SyllabifiedCandidate);

impl Constraint for Max {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let mut violations = (self.0.form.len() - surface.form.len()) * 3;

        if !surface.form.is_empty()
            && (surface.form[0].char != self.0.form[0].char
                || surface.form[surface.form.len() - 1].char
                    != self.0.form[self.0.form.len() - 1].char)
        {
            violations += 1;
        }

        violations
    }
}

#[derive(Debug)]
pub struct MaxInitialV(pub SyllabifiedCandidate);

impl Constraint for MaxInitialV {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let underlying_init = self
            .0
            .form
            .iter()
            .filter(|seg| seg.morpheme_index == UnderlyingIndex::Initial)
            .count();

        let surface_init = surface
            .form
            .iter()
            .filter(|seg| seg.morpheme_index == UnderlyingIndex::Initial)
            .count();

        underlying_init - surface_init
    }
}

#[derive(Debug)]
pub struct MaxFinalV(pub SyllabifiedCandidate);

impl Constraint for MaxFinalV {
    fn evaluate(&self, surface: SyllabifiedCandidate) -> usize {
        let underlying_final = self
            .0
            .form
            .iter()
            .filter(|seg| seg.morpheme_index == UnderlyingIndex::Final)
            .count();

        let surface_final = surface
            .form
            .iter()
            .filter(|seg| seg.morpheme_index == UnderlyingIndex::Final)
            .count();

        underlying_final - surface_final
    }
}
