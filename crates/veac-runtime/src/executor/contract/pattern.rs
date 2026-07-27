mod passlog;

pub(in crate::executor) use passlog::{
    accepts as passlog_accepts, overlap as passlog_overlap,
    overlap_pattern as passlog_overlap_pattern,
};
