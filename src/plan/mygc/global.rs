use crate::plan::global::BasePlan;
use crate::plan::global::CreateGeneralPlanArgs;
use crate::plan::global::CreateSpecificPlanArgs;
use crate::plan::mygc::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::immortalspace::ImmortalSpace;
use crate::policy::space::Space;
use crate::scheduler::GCWorkScheduler;
use crate::util::alloc::allocators::AllocatorSelector;
#[allow(unused_imports)]
use crate::util::heap::VMRequest;
use crate::util::metadata::side_metadata::{SideMetadataContext, SideMetadataSanity};
use crate::util::opaque_pointer::*;
use crate::vm::VMBinding;
use enum_map::EnumMap;


use crate::policy::immortalspace::ImmortalSpace as MyGCImmortalSpace;


pub struct MyGC<VM: VMBinding> {
    pub base: BasePlan<VM>,
    pub mygc_space: MyGCImmortalSpace<VM>,
    pub immortal: ImmortalSpace<VM>,
    pub los: ImmortalSpace<VM>,
}

pub const MYGC_CONSTRAINTS: PlanConstraints = PlanConstraints {
    moves_objects: true,
    gc_header_bits: 2,
    gc_header_words: 0,
    num_specialized_scans: 1,
    ..PlanConstraints::default()
};

impl<VM: VMBinding> Plan for MyGC<VM> {
    type VM = VM;

    fn constraints(&self) -> &'static PlanConstraints {
        &MyGC_CONSTRAINTS
    }

    fn get_spaces(&self) -> Vec<&dyn Space<Self::VM>> {
        let mut ret = self.base.get_spaces();
        ret.push(&self.mygc_space);
        ret.push(&self.immortal);
        ret.push(&self.los);
        ret
    }

    fn collection_required(&self, space_full: bool, _space: Option<&dyn Space<Self::VM>>) -> bool {
        self.base().collection_required(self, space_full)
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.base
    }

    fn prepare(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn release(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn get_allocator_mapping(&self) -> &'static EnumMap<AllocationSemantics, AllocatorSelector> {
        &ALLOCATOR_MAPPING
    }

    fn schedule_collection(&'static self, _scheduler: &GCWorkScheduler<VM>) {
        unreachable!("GC triggered in mygc")
    }

    fn get_used_pages(&self) -> usize {
        self.mygc_space.reserved_pages()
            + self.immortal.reserved_pages()
            + self.los.reserved_pages()
            + self.base.get_used_pages()
    }

    fn handle_user_collection_request(
        &self,
        _tls: VMMutatorThread,
        _force: bool,
        _exhaustive: bool,
    ) {
        warn!("User attempted a collection request, but it is not supported in MyGC. The request is ignored.");
    }
}

impl<VM: VMBinding> MyGC<VM> {
    pub fn new(args: CreateGeneralPlanArgs<VM>) -> Self {
        let mut plan_args = CreateSpecificPlanArgs {
            global_args: args,
            constraints: &MyGC_CONSTRAINTS,
            global_side_metadata_specs: SideMetadataContext::new_global_specs(&[]),
        };

        let res = MyGC {
            mygc_space: MyGCImmortalSpace::new(plan_args.get_space_args(
                "mygc_space",
                cfg!(not(feature = "mygc_no_zeroing")),
                VMRequest::discontiguous(),
            )),
            immortal: ImmortalSpace::new(plan_args.get_space_args(
                "immortal",
                true,
                VMRequest::discontiguous(),
            )),
            los: ImmortalSpace::new(plan_args.get_space_args(
                "los",
                true,
                VMRequest::discontiguous(),
            )),
            base: BasePlan::new(plan_args),
        };

        // Use SideMetadataSanity to check if each spec is valid. This is also needed for check
        // side metadata in extreme_assertions.
        let mut side_metadata_sanity_checker = SideMetadataSanity::new();
        res.base()
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        res.mygc_space
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);

        res
    }
}
