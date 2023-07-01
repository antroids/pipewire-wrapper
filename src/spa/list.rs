use std::marker::PhantomData;
use std::ptr::{addr_of, null, null_mut};

use spa_sys::spa_list;

use pipewire_proc_macro::RawWrapper;

use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ListRef {
    #[raw]
    raw: spa_sys::spa_list,
}

pub trait ListElement {
    fn as_list_ptr(&self) -> *mut spa_list;
    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self;

    unsafe fn init_detached(&mut self) {
        let list = self.as_list_ptr();
        (*list).prev = list;
        (*list).next = list;
    }

    unsafe fn remove(&mut self) {
        let list = self.as_list_ptr();

        (*(*list).prev).next = (*list).next;
        (*(*list).next).prev = (*list).prev;
    }
}

impl ListElement for ListRef {
    fn as_list_ptr(&self) -> *mut spa_list {
        self.as_raw_ptr()
    }

    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self {
        ptr as *mut Self
    }
}

pub trait List
where
    Self: Sized,
{
    type Elem: ListElement;

    fn as_list_ptr(&self) -> *mut spa_list;
    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self;

    fn init(&mut self) {
        unsafe {
            let list = self.as_list_ptr();
            (*list).next = list;
            (*list).prev = list;
        }
    }

    fn initialized(&self) -> bool {
        unsafe {
            let list = self.as_list_ptr();
            !(*list).prev.is_null()
        }
    }

    fn empty(&self) -> bool {
        let list = self.as_list_ptr();
        unsafe {
            (*list).next == (*list).prev
                && (*list).next as *const spa_list == list as *const spa_list
        }
    }

    unsafe fn insert(&mut self, elem: &mut Self::Elem) {
        let list = self.as_list_ptr();
        let elem = elem.as_list_ptr();
        (*elem).prev = list;
        (*elem).next = (*list).next;
        (*list).next = elem;
        (*(*elem).next).prev = elem;
    }

    unsafe fn insert_list(&mut self, other: &mut Self) {
        if !other.empty() {
            let list = self.as_list_ptr();
            let other = other.as_list_ptr();
            (*(*other).next).prev = list;
            (*(*other).prev).next = (*list).next;
            (*(*list).next).prev = (*other).prev;
            (*list).next = (*other).next;
        }
    }

    unsafe fn next(&self, current: &Self::Elem) -> Option<&mut Self::Elem> {
        let link = (*current.as_list_ptr()).next;
        if link == self.as_list_ptr() {
            None
        } else {
            Some(&mut *Self::Elem::from_list_ptr(link))
        }
    }

    unsafe fn prev(&self, current: &Self::Elem) -> Option<&mut Self::Elem> {
        let link = (*current.as_list_ptr()).prev;
        if link == self.as_list_ptr() {
            None
        } else {
            Some(&mut *Self::Elem::from_list_ptr(link))
        }
    }

    unsafe fn first(&self) -> Option<&mut Self::Elem> {
        let list = self.as_list_ptr();
        let first = (*list).next;
        if first == list {
            None
        } else {
            Some(&mut *Self::Elem::from_list_ptr(first))
        }
    }

    unsafe fn last(&self) -> Option<&mut Self::Elem> {
        let list = self.as_list_ptr();
        let last = (*list).prev;
        if last == list {
            None
        } else {
            Some(&mut *Self::Elem::from_list_ptr(last))
        }
    }

    unsafe fn append(&mut self, elem: &mut Self::Elem) {
        let list = (*self.as_list_ptr()).prev;
        let elem = elem.as_list_ptr();
        (*elem).prev = list;
        (*elem).next = (*list).next;
        (*list).next = elem;
        (*(*elem).next).prev = elem;
    }

    unsafe fn prepend(&mut self, elem: &mut Self::Elem) {
        self.insert(elem)
    }

    unsafe fn clean(&mut self) {
        while let Some(first) = self.first() {
            first.remove()
        }
    }

    unsafe fn iter(&self) -> ListIterator<Self> {
        ListIterator {
            list: self,
            element: None,
        }
    }

    unsafe fn iter_mut(&self) -> ListMutIterator<Self> {
        ListMutIterator {
            list: self,
            element: None,
        }
    }
}

pub struct ListIterator<'l, L: List> {
    list: &'l L,
    element: Option<&'l L::Elem>,
}

impl<'l, L: List> Iterator for ListIterator<'l, L> {
    type Item = &'l L::Elem;

    fn next(&mut self) -> Option<Self::Item> {
        let next_ptr = if let Some(element) = self.element {
            unsafe { (*element.as_list_ptr()).next }
        } else {
            unsafe { (*self.list.as_list_ptr()).next }
        };
        if next_ptr == self.list.as_list_ptr() {
            None
        } else {
            self.element = unsafe { Some(&*(L::Elem::from_list_ptr(next_ptr))) };
            self.element
        }
    }
}

pub struct ListMutIterator<'l, L: List> {
    list: &'l L,
    element: Option<*mut spa_list>,
}

impl<'l, L: List> Iterator for ListMutIterator<'l, L> {
    type Item = &'l mut L::Elem;

    fn next(&mut self) -> Option<Self::Item> {
        let next_ptr = if let Some(element) = self.element {
            unsafe { (*element).next }
        } else {
            unsafe { (*self.list.as_list_ptr()).next }
        };
        if next_ptr == self.list.as_list_ptr() {
            None
        } else {
            self.element = Some(next_ptr);
            unsafe { Some(&mut *(L::Elem::from_list_ptr(next_ptr))) }
        }
    }
}

#[repr(C)]
#[cfg(test)]
#[derive(RawWrapper)]
struct TestList {
    #[raw]
    list: spa_list,
}

#[repr(C)]
#[cfg(test)]
#[derive(Debug)]
struct TestElement {
    link: spa_list,
    payload: u32,
}

#[cfg(test)]
impl ListElement for TestElement {
    fn as_list_ptr(&self) -> *mut spa_list {
        addr_of!(self.link) as *mut _
    }

    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self {
        ptr as *mut Self
    }
}

#[cfg(test)]
impl List for TestList {
    type Elem = TestElement;

    fn as_list_ptr(&self) -> *mut spa_list {
        addr_of!(self.list) as *mut _
    }

    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self {
        ptr as *mut Self
    }
}

#[cfg(test)]
impl TestElement {
    fn new(payload: u32) -> Self {
        Self {
            link: spa_list {
                next: null_mut(),
                prev: null_mut(),
            },
            payload,
        }
    }
}

#[cfg(test)]
impl Default for TestList {
    fn default() -> Self {
        Self {
            list: spa_list {
                next: null_mut(),
                prev: null_mut(),
            },
        }
    }
}

#[test]
fn test_init() {
    let mut list = TestList::default();

    unsafe {
        assert!(!list.initialized());
        list.init();
        assert!(list.initialized());
    }
}

#[test]
fn test_empty() {
    let mut list = TestList::default();
    unsafe {
        list.init();
        assert!(list.empty());
    }
}

#[test]
fn test_insert() {
    let mut list = TestList::default();

    unsafe {
        list.init();
        let mut elem1 = TestElement::new(1);
        let mut elem2 = TestElement::new(2);
        assert!(list.empty());

        list.insert(&mut elem1);

        assert!(!list.empty());
        assert_eq!(list.first().unwrap().payload, elem1.payload);
        assert!(list.next(&elem1).is_none());
        assert!(list.prev(&elem1).is_none());

        list.insert(&mut elem2);
        assert_eq!(list.first().unwrap().payload, elem2.payload);
        assert_eq!(list.next(&elem2).unwrap().payload, elem1.payload);
        assert!(list.next(&elem1).is_none());
    }
}

#[test]
fn test_insert_list() {
    let mut list1 = TestList::default();
    let mut list2 = TestList::default();

    unsafe {
        list1.init();
        list2.init();
        let mut elem1 = TestElement::new(1);
        let mut elem2 = TestElement::new(2);
        let mut elem3 = TestElement::new(3);
        let mut elem4 = TestElement::new(4);

        list1.insert(&mut elem1);
        list1.insert(&mut elem2);

        list2.insert(&mut elem3);
        list2.insert(&mut elem4);

        list1.insert_list(&mut list2);

        let current = list1.first();
        assert_eq!(current.as_ref().unwrap().payload, 4);
        let current = list1.next(current.unwrap());
        assert_eq!(current.as_ref().unwrap().payload, 3);
        let current = list1.next(current.unwrap());
        assert_eq!(current.as_ref().unwrap().payload, 2);
        let current = list1.next(current.unwrap());
        assert_eq!(current.as_ref().unwrap().payload, 1);
        let current = list1.next(current.unwrap());
        assert!(current.is_none());
    }
}

#[test]
fn test_iterator() {
    let mut list1 = TestList::default();
    unsafe {
        let mut elem1 = TestElement::new(1);
        let mut elem2 = TestElement::new(2);
        let mut elem3 = TestElement::new(3);
        let mut elem4 = TestElement::new(4);

        list1.init();
        list1.insert(&mut elem1);
        list1.insert(&mut elem2);
        list1.insert(&mut elem3);
        list1.insert(&mut elem4);

        let mut iter = list1.iter();

        assert_eq!(iter.next().unwrap().payload, 4);
        assert_eq!(iter.next().unwrap().payload, 3);
        assert_eq!(iter.next().unwrap().payload, 2);
        assert_eq!(iter.next().unwrap().payload, 1);

        assert!(iter.next().is_none());
    }
}

#[test]
fn test_mut_iterator() {
    let mut list1 = TestList::default();
    unsafe {
        let mut elem1 = TestElement::new(1);
        let mut elem2 = TestElement::new(2);
        let mut elem3 = TestElement::new(3);
        let mut elem4 = TestElement::new(4);

        list1.init();
        list1.insert(&mut elem1);
        list1.insert(&mut elem2);
        list1.insert(&mut elem3);
        list1.insert(&mut elem4);

        let mut iter = list1.iter_mut();

        assert_eq!(iter.next().unwrap().payload, 4);
        assert_eq!(iter.next().unwrap().payload, 3);
        assert_eq!(iter.next().unwrap().payload, 2);
        assert_eq!(iter.next().unwrap().payload, 1);

        assert!(iter.next().is_none());
    }
}
