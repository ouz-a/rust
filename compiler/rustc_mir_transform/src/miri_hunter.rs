use crate::add_call_guards::AddCallGuards;
use crate::deref_separator::Derefer;
use crate::elaborate_drops::ElaborateDrops;
use crate::fake_drop::FakeDrops;
use rustc_middle::mir::pretty::write_mir_fn;
use rustc_middle::mir::*;
use rustc_middle::ty::print::with_no_trimmed_paths;
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
        let fake_elab = FakeDrops {};
        let call_g = AddCallGuards::CriticalCallEdges;
        // derefer before elaborate
        derefer.run_pass(tcx, &mut og_bod);
        call_g.run_pass(tcx, &mut og_bod);
        fake_elab.run_pass(tcx, &mut og_bod);
        trace!("DEREF DEREF DEREDEREFDEREFDEREFDEREFF");
        // derefer after elaborate
        call_g.run_pass(tcx, &mut clon);
        elab.run_pass(tcx, &mut clon);
        derefer.run_pass(tcx, &mut clon);
        trace!("ELAB ELAB ELAB ELAB ELAB ELAB");

        let deref_before_elab = count_drop(&mut og_bod);
        trace!("--------------deref before elab up-------------");
        let deref_after_elab = count_drop(&mut clon);
        if deref_before_elab != deref_after_elab {
            let mut a = Vec::new();
            let mut f = Vec::new();
            with_no_trimmed_paths!({
                trace!("d-b {}   d-a {} ", deref_before_elab, deref_after_elab);
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                trace!("--------------------------------------------------");
                write_mir_fn(tcx, &og_bod, &mut |_, _| Ok(()), &mut a).unwrap();
                write_mir_fn(tcx, &clon, &mut |_, _| Ok(()), &mut f).unwrap();
                let pop = String::from_utf8_lossy(&a);
                if !pop.contains("syn") {
                    trace!("deref before elab {}", deref_before_elab);
                    trace!("deref after elab {}", deref_after_elab);
                    trace!("deref then elab {}", String::from_utf8_lossy(&a));
                    trace!("elab then deref {}", String::from_utf8_lossy(&f));
                    //span_bug!(og_bod.span, "og bod");
                }
            });
        }
        trace!("------------end------------");
    }
}
