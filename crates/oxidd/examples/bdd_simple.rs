use oxidd::bdd::BDDFunction;
use oxidd::util::AllocResult;
use oxidd::{BooleanFunction, Manager, ManagerRef};
use oxidd_dump::dot::dump_all;

// fn main() -> AllocResult<()> {
//     let manager_ref = oxidd::bdd::new_manager(1024, 1024, 1);
//     let (x1, x2, x3) = manager_ref.with_manager_exclusive(|manager| {
//         manager.add_named_vars(["x", "y", "z"]).unwrap();
//         Ok((
//             BDDFunction::var(manager, 0)?,
//             BDDFunction::var(manager, 1)?,
//             BDDFunction::var(manager, 2)?,
//         ))
//     })?;

//     let res = x1.and(&x2)?.or(&x3)?;
//     println!("{}", res.satisfiable());

//     manager_ref.with_manager_shared(|manager| {
//         let file =
//             std::fs::File::create("bdd_simple.dot").expect("could not create
// `bdd_simple.dot`");         dump_all(file, manager, [(&res, "(x and y) or
// z")]).expect("dot export failed");     });
//     Ok(())
// }

fn main() -> AllocResult<()> {
    let manager_ref = oxidd::bdd::new_manager(1024, 1024, 1);
    let (a, b, c, d) = manager_ref.with_manager_exclusive(|manager| {
        manager.add_named_vars(["a", "b", "c", "d"]).unwrap();
        Ok((
            BDDFunction::var(manager, 0)?,
            BDDFunction::var(manager, 1)?,
            BDDFunction::var(manager, 2)?,
            BDDFunction::var(manager, 3)?,
        ))
    })?;

    let mt0 = a.and(&b)?;
    let mt1 = a.and(&c)?;
    let mt2 = a.and(&d)?;
    let mt3 = b.and(&c)?.and(&d)?;
    let res = mt0.or(&mt1)?.or(&mt2)?.or(&mt3)?;
    println!("{}", res.satisfiable());

    manager_ref.with_manager_shared(|manager| {
        let file =
            std::fs::File::create("bdd_simple.dot").expect("could not create `bdd_simple.dot`");
        dump_all(file, manager, [(&res, "maj2")]).expect("dot export failed");
    });
    Ok(())
}
