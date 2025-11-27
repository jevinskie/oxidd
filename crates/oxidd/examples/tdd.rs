use oxidd::tdd::new_manager;
use oxidd::tdd::TDDFunction;
use oxidd::util::AllocResult;
use oxidd::ManagerRef;
use oxidd::TVLFunction;
use oxidd_core::Manager;
use oxidd_dump::dot::dump_all;

fn main() -> AllocResult<()> {
    let manager_ref = new_manager(1024, 1024, 1);
    let [a, b, c, d] = manager_ref.with_manager_exclusive(|manager| {
        manager.add_named_vars(["a", "b", "c", "d"]).unwrap();
        Ok([
            TDDFunction::var(manager, 0)?,
            TDDFunction::var(manager, 1)?,
            TDDFunction::var(manager, 2)?,
            TDDFunction::var(manager, 3)?,
        ])
    })?;

    manager_ref.with_manager_shared(|manager| {
        let mt0 = a.and(&b)?;
        let mt1 = a.and(&c)?;
        let mt2 = a.and(&d)?;
        let mt3 = b.and(&c)?.and(&d)?;
        let maj2 = mt0.or(&mt1)?.or(&mt2)?.or(&mt3)?;

        manager.gc();

        let file = std::fs::File::create("maj2.dot").expect("could not create `maj2.dot`");
        dump_all(file, manager, [(&maj2, "maj2")]).expect("dot export failed");
        Ok(())
    })
}
