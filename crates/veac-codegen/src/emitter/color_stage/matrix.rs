use veac_plan::canonical::RgbMatrixAdjustment;

use super::{filter, number};
use crate::emitter::EmitContext;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    input: String,
    value: RgbMatrixAdjustment,
) -> String {
    let input = filter(context, input, "format=gbrapf32le".to_owned(), "matrixfmtv");
    let channel = |row: usize, offset: usize| {
        let start = row * 3;
        format!(
            "clip(r(X\\,Y)*{}+g(X\\,Y)*{}+b(X\\,Y)*{}+{}\\,0\\,1)",
            number(value.matrix[start]),
            number(value.matrix[start + 1]),
            number(value.matrix[start + 2]),
            number(value.offset[offset])
        )
    };
    filter(
        context,
        input,
        format!(
            "geq=r='{}':g='{}':b='{}':a='alpha(X\\,Y)'",
            channel(0, 0),
            channel(1, 1),
            channel(2, 2)
        ),
        "matrixv",
    )
}
