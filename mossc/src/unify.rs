
use std::collections::HashMap;
use std::rc::Rc;

use super::{Type, TypeId, FnType, PolyType};

fn type_mismatch(expected: &Type, given: &Type) -> String {
    format!("\n    expected type {}\n    found    type {}", expected, given)
}

pub struct Substitution {
    pub map: HashMap<TypeId, Type>
}
impl Substitution {
    pub fn new() -> Self {
        Self {map: HashMap::new()}
    }

    pub fn apply(&self, typ: &Type) -> Type {
        match typ {
            Type::None => Type::None,
            Type::Atom(typ) => {
                Type::Atom(typ.clone())
            },
            Type::Var(id) => {
                let subs = match self.map.get(id) {
                    Some(value) => value,
                    None => return Type::Var(*id)
                };
                if let Type::Atom(typ) = subs {
                    Type::Atom(typ.clone())
                } else if subs.contains_var() {
                    self.apply(subs)
                } else {
                    subs.clone()
                }
            },
            Type::App(app) => {
                Type::app(app.iter().map(|x|
                    self.apply(x)).collect::<Vec<Type>>())
            },
            Type::Fn(typ) => {
                Type::Fn(Rc::new(FnType {
                    argc_min: typ.argc_min,
                    argc_max: typ.argc_max,
                    arg_self: self.apply(&typ.arg_self),
                    arg: typ.arg.iter().map(|x| self.apply(x)).collect(),
                    ret: self.apply(&typ.ret)
                }))
            },
            Type::Poly(poly_type) => {
                Type::Poly(Rc::new(PolyType {
                    variables: poly_type.variables.clone(),
                    scheme: self.apply(&poly_type.scheme)
                }))
            }
        }
    }

    fn unify_fn(&mut self, f1: &FnType, f2: &FnType,
        log: &mut Option<&mut String>
    ) -> Result<(), ()>
    {
        self.unify(&f1.ret, &f2.ret, log)?;
        if f1.arg.len() != f2.arg.len() ||
           f1.argc_min != f2.argc_min ||
           f1.argc_max != f2.argc_max
        {
            if let Some(log) = log {
                **log = format!(
                    "mismatch in number of arguments:\n  expected {}\n  found {}",
                    f1.arg.len(), f2.arg.len()
                );
            }
            return Err(());
        }
        self.unify(&f1.arg_self, &f2.arg_self, log)?;
        for i in 0..f1.arg.len() {
            self.unify(&f1.arg[i], &f2.arg[i], log)?;
        }
        Ok(())
    }

    pub fn unify_var(&mut self, tv: TypeId, t2: &Type,
        log: &mut Option<&mut String>
    ) -> Result<(), ()>
    {
        // println!("{} = {}", tv, t2);
        if let Some(t1) = self.map.get(&tv) {
            let t1 = t1.clone();
            return self.unify(&t1, t2, log);
        } else if let Type::Var(tv2) = t2 {
            if tv == *tv2 {return Ok(());}
            if let Some(t2) = self.map.get(tv2) {
                let t2 = t2.clone();
                return self.unify_var(tv, &t2, log);
            }
        }
        self.map.insert(tv, t2.clone());
        Ok(())
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type,
        log: &mut Option<&mut String>
    ) -> Result<(), ()>
    {
        if let Type::Var(tv1) = t1 {
            return self.unify_var(*tv1, t2, log);
        }
        if let Type::Var(tv2) = t2 {
            return self.unify_var(*tv2, t1, log);
        }
        match t1 {
            Type::Atom(t1) => {
                if let Type::Atom(t2) = t2 {
                    if Rc::ptr_eq(t1, t2) {return Ok(());}
                }
            },
            Type::App(app_t1) => {
                if let Type::App(app_t2) = t2 {
                    if app_t1.len() != app_t2.len() {
                        if let Some(log) = log {
                            **log = format!(
                                "Mismatch in number of type arguments: \n  expected {}, \n  found {}",
                                t1, t2
                            );
                        }
                        return Err(());
                    }
                    for i in 0..app_t1.len() {
                        self.unify(&app_t1[i], &app_t2[i], log)?;
                    }
                    return Ok(());
                } else {
                    if let Some(log) = log {
                        **log = type_mismatch(t1, t2);
                    }
                    return Err(());
                }
            },
            Type::Fn(fn_t1) => {
                if let Type::Fn(fn_t2) = t2 {
                    return self.unify_fn(fn_t1, fn_t2, log);
                } else {
                    if let Some(log) = log {
                        **log = type_mismatch(t1, t2);
                    }
                    return Err(());
                }
            },
            t1 => {
                if let Some(log) = log {
                    **log = format!("Cannot unify {}", t1);
                }
                return Err(());
            }
        }
        if let Some(log) = log {
            **log = type_mismatch(t1, t2);
        }
        Err(())
    }
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut a: Vec<(&TypeId, &Type)> = self.map.iter().collect();
        a.sort_by_key(|(id, _)| id.0);
        for (key,value) in &a {
            writeln!(f, "{} := {},", key.0, value).ok();
        }
        Ok(())
    }
}

