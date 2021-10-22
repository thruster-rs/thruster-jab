use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[macro_export]
macro_rules! provide {
    ($jab_state:expr, dyn $trait:tt, $value:expr) => {
        let _temp: std::boxed::Box<dyn $trait + Send + Sync> = Box::new($value);

        $jab_state.put(_temp);
    };
    ($jab_state:expr, $trait:ty, $value:expr) => {
        let _temp: std::boxed::Box<$trait> = Box::new($value);

        $jab_state.put(_temp);
    };
    ($jab_state:expr, $value:expr) => {
        $jab_state.put(Box::new($value));
    };
}

#[macro_export]
macro_rules! fetch {
    ($jab_state:expr, dyn $trait:tt) => {
        $jab_state.get::<Box<dyn $trait + Send + Sync>>()
    };
    ($jab_state:expr, $trait:ty) => {
        $jab_state.get::<Box<$trait>>()
    };
}

trait JabStateWithDI {
    fn get_mut<'a>(&'a mut self) -> &'a mut JabDI;
    fn get<'a>(&'a self) -> &'a JabDI;
}

#[derive(Debug, Default)]
pub struct JabDI {
    dep_map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl JabDI {
    pub fn put<T: 'static + Send + Sync>(&mut self, val: T) {
        self.dep_map
            .insert(TypeId::of::<T>(), Box::new(Box::new(val)));
    }

    pub fn get<T: 'static + ?Sized>(&self) -> &T {
        if let Some(v) = self.try_get() {
            v
        } else {
            panic!("Could not find requested type");
        }
    }

    pub fn try_get<T: 'static + ?Sized>(&self) -> Option<&T> {
        if let Some(dep) = self.dep_map.get(&TypeId::of::<T>()) {
            if let Some(val) = dep.downcast_ref::<Box<T>>() {
                return Some(val);
            }
        }

        None
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        if let Some(v) = self.try_get_mut() {
            v
        } else {
            panic!("Could not find requested type");
        }
    }

    pub fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(dep) = self.dep_map.get_mut(&TypeId::of::<T>()) {
            if let Some(val) = Box::new(dep).downcast_mut::<T>() {
                return Some(val);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::JabDI;

    #[derive(Debug, PartialEq)]
    struct A(i32);

    #[derive(Debug, PartialEq)]
    struct B(i32);

    trait C {
        fn valc(&self) -> i32;
    }
    trait D {
        fn vald(&self) -> i32;
    }

    impl C for A {
        fn valc(&self) -> i32 {
            self.0
        }
    }

    impl D for B {
        fn vald(&self) -> i32 {
            self.0
        }
    }

    #[test]
    fn test_get_struct() {
        let mut jab = JabDI::default();

        let a = A(0);
        let b = B(1);

        provide!(jab, a);
        provide!(jab, b);

        assert_eq!(
            0,
            fetch!(jab, A).0,
            "it should correctly find struct A for struct A"
        );

        assert_eq!(
            1,
            fetch!(jab, B).0,
            "it should correctly find struct B for struct B"
        );
    }

    #[test]
    fn test_get_trait() {
        let mut jab = JabDI::default();

        let a = A(0);
        let b = B(1);

        provide!(jab, dyn C, a);
        provide!(jab, dyn D, b);

        assert_eq!(
            0,
            fetch!(jab, dyn C).valc(),
            "it should correctly find struct A for trait C"
        );

        assert_eq!(
            1,
            fetch!(jab, dyn D).vald(),
            "it should correctly find struct B for trait D"
        );
    }
}
