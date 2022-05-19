use crate::add_call_guards::AddCallGuards;
use crate::deref_separator::Derefer;
use crate::elaborate_drops::ElaborateDrops;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

pub struct MiriHunter;

pub fn count_drop<'tcx>(body: &mut Body<'tcx>) -> i32 {
    let mut drop_count = 0;
    for (_bb, data) in body.basic_blocks_mut().iter_enumerated_mut() {
        if let Some(term) = &data.terminator {
            if let TerminatorKind::Drop { .. } = term.kind {
                drop_count += 1
            }
        }
    }
    drop_count
}

impl<'tcx> MirPass<'tcx> for MiriHunter {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let mut og_bod = body.clone();
        let mut clon = og_bod.clone();
        let derefer = Derefer {};
        let elab = ElaborateDrops {};
        let call_g = AddCallGuards::CriticalCallEdges;
        // derefer before elaborate
        println!("--------derefer then elaborate-----------------");
        derefer.run_pass(tcx, &mut og_bod);
        call_g.run_pass(tcx, &mut og_bod);
        elab.run_pass(tcx, &mut og_bod);
        // derefer after elaborate
        println!("--------elaborate then derefer-----------------");
        call_g.run_pass(tcx, &mut clon);
        elab.run_pass(tcx, &mut clon);
        derefer.run_pass(tcx, &mut clon);

        let deref_before_elab = count_drop(&mut og_bod);
        let deref_after_elab = count_drop(&mut clon);
        if deref_before_elab != deref_after_elab {
            println!("og.body {:#?}", og_bod);
            println!("clon body {:#?}", clon);
            span_bug!(og_bod.span, "og bod");
        }
    }
}
