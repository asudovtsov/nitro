
// #[derive(Debug, Copy, Clone)]
// pub(crate) struct Strict<T> {
//     non_null: NonNull<T>
// }

// impl<T> Strict<T> {
//     pub fn new(ptr: *mut T) -> Option<Strict<T>> {
//         if !ptr.is_null() /* && ptr.is_aligned() #TODO */ {
//             Some(Strict{non_null: unsafe { NonNull::new_unchecked(ptr) }})
//         } else {
//             None
//         }
//     }

//     pub fn as_ref(&self) -> &T {
//         unsafe { self.non_null.as_ref() }
//     }

//     pub fn as_mut(&mut self) -> &mut T {
//         unsafe { self.non_null.as_mut() }
//     }

//     pub fn as_ptr(&mut self) -> *mut T {
//         self.non_null.as_ptr()
//     }
// }

// impl<T> Eq for Strict<T> {}

// impl<T> PartialEq for Strict<T> {
//     #[inline]
//     fn eq(&self, other: &Self) -> bool {
//         self.non_null == other.non_null
//     }
// }

// impl<T> fmt::Pointer for Strict<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Pointer::fmt(&self.non_null, f)
//     }
// }