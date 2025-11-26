use oxidd::tdd::TDDFunction;
use oxidd::ManagerRef;
use oxidd::TVLFunction;
use oxidd_core::Manager;
use oxidd_dump::dot::dump_all;

fn main() {
    let manager_ref = oxidd::tdd::new_manager(1024, 1024, 1);
    let (x1, x2, x3, x4) = manager_ref.with_manager_exclusive(|manager| {
        (
        manager.add_named_vars(["x", "y", "z", "u"]).unwrap();
        Ok((
            TDDFunction::var(manager,0).unwrap()?,
            TDDFunction::var(manager,1).unwrap()?,
            TDDFunction::var(manager,2).unwrap()?,
            TDDFunction::var(manager,3).unwrap()?,)
        )?)
    });

    manager_ref.with_manager_shared(|manager| {
        let res = x1.and(&x2).unwrap().or(&x4).unwrap();

        manager.gc();

        let file = std::fs::File::create("tdd.dot").expect("could not create `tdd.dot`");
        dump_all(file, manager, [(&res, "(x1 ∧ x2) ∨ x4")]).expect("dot export failed");
    });
}
