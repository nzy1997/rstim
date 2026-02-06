#[derive(Debug, Clone, PartialEq)]
pub enum StimTarget {
    Qubit(u32),
    Rec(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StimInstr {
    pub name: String,
    pub tag: Option<String>,
    pub args: Vec<f64>,
    pub targets: Vec<StimTarget>,
}

impl StimInstr {
    pub fn new(name: &str, args: Vec<f64>, targets: Vec<StimTarget>) -> Self {
        Self {
            name: name.to_string(),
            tag: None,
            args,
            targets,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub coords: Vec<f64>,
    pub rec_offsets: Vec<i32>,
    pub observable_index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    Detector,
    ObservableInclude,
}

impl Annotation {
    pub fn detector(coords: Vec<f64>, rec_offsets: Vec<i32>) -> Self {
        Self {
            kind: AnnotationKind::Detector,
            coords,
            rec_offsets,
            observable_index: None,
        }
    }

    pub fn observable_include(index: u32, rec_offsets: Vec<i32>) -> Self {
        Self {
            kind: AnnotationKind::ObservableInclude,
            coords: vec![],
            rec_offsets,
            observable_index: Some(index),
        }
    }
}
