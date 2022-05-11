use crate::add_call_guards::AddCallGuards;
use crate::deref_separator::Derefer;
use crate::elaborate_drops::ElaborateDrops;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;

pub struct MiriHunter;

impl<'tcx> MirPass<'tcx> for MiriHunter {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let mut og_bod = body.clone();
        let mut clon = og_bod.clone();
        let derefer = Derefer {};
        let elab = ElaborateDrops {};
        let call_g = AddCallGuards::CriticalCallEdges;
        // derefer before elaborate
        derefer.run_pass(tcx, &mut og_bod);
        call_g.run_pass(tcx, &mut og_bod);
        elab.run_pass(tcx, &mut og_bod);
        // derefer after elaborate
        call_g.run_pass(tcx, &mut clon);
        elab.run_pass(tcx, &mut clon);
        derefer.run_pass(tcx, &mut clon);

        let og_str = format!("{:?}", og_bod);
        let clon_str = format!("{:?}", clon);
        if og_str != clon_str && og_str.len() != clon_str.len() {
            println!("og.body {:#?}", og_bod);
            println!("clon body {:#?}", clon);
            span_bug!(og_bod.span, "og bod");
        }
    }
}
