use super::global::Ng1GC;
use crate::policy::gc_work::DEFAULT_TRACE;
use crate::scheduler::gc_work::PlanProcessEdges;
use crate::vm::VMBinding;

pub struct Ng1GCWorkContext<VM: VMBinding>(std::marker::PhantomData<VM>);
impl<VM: VMBinding> crate::scheduler::GCWorkContext for Ng1GCWorkContext<VM> {
    type VM = VM;
    type PlanType = Ng1GC<VM>;
    type ProcessEdgesWorkType = PlanProcessEdges<Self::VM, Ng1GC<VM>, DEFAULT_TRACE>;
}
