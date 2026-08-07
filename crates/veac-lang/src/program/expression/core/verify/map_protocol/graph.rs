use crate::program::expression::core::{CoreProgram, CoreTerminator};

pub(super) fn postdominates(program: &CoreProgram, definition: usize, usage: usize) -> bool {
    if definition == usage {
        return true;
    }
    let mut seen = vec![false; program.blocks.len()];
    let mut pending = successors(&program.blocks[definition].terminator);
    while let Some(block) = pending.pop() {
        if block == usage || std::mem::replace(&mut seen[block], true) {
            continue;
        }
        match &program.blocks[block].terminator {
            CoreTerminator::Return { .. } => return false,
            terminator => pending.extend(successors(terminator)),
        }
    }
    true
}

fn successors(terminator: &CoreTerminator) -> Vec<usize> {
    match terminator {
        CoreTerminator::Return { .. } => Vec::new(),
        CoreTerminator::Jump { target, .. } => {
            vec![target.index().expect("verified jump target")]
        }
        CoreTerminator::Branch {
            then_target,
            else_target,
            ..
        } => vec![
            then_target.index().expect("verified branch target"),
            else_target.index().expect("verified branch target"),
        ],
        CoreTerminator::Match { arms, .. } => arms
            .iter()
            .map(|arm| arm.target().index().expect("verified match target"))
            .collect(),
        CoreTerminator::ForEach(value) => vec![value
            .continuation()
            .index()
            .expect("verified for-each continuation")],
    }
}
