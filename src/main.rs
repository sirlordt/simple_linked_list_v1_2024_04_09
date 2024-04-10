use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use std::fmt::Debug;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

#[derive(Debug)]
struct Node<T: Debug> {
    data: T,
    next: Link<T>,
}

struct LinkedList<T: Debug> {
    head: Link<T>,
    tail: Link<T>,
}

pub struct Iter<'a, T: Debug + 'a> {
    current: Link<T>,
    marker: PhantomData<&'a Node<T>>, //A trick needed to allow return &T. See the Iter.next() method
}

pub struct IterMut<'a, T: Debug + 'a> {
    current: Link<T>,
    marker: PhantomData<&'a mut Node<T>>, //A trick needed to allow return &mut T. See the ItemMut.next() method
}

impl<T: Debug> Node<T> {
    fn new(data: T) -> Self {
        Self { data, next: None }
    }
}

impl<T: Debug> LinkedList<T> {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    fn push_back(&mut self, data: T) {
        let new_node = Rc::new(RefCell::new(Node::new(data)));

        match self.tail {
            None => self.tail = Some(new_node),
            Some(_) => {
                let new_tail = Some(new_node);
                self.tail.as_ref().unwrap().borrow_mut().next = new_tail.clone(); //1. Safe Method 1

                // let x = unsafe{ &mut *self.tail.as_ref().unwrap().as_ptr() }; //1. Unsafe Method 2
                // x.next = new_tail.clone();                                    //1. Unsafe Method 2

                //unsafe{ (&mut *self.tail.as_ref().unwrap().as_ptr()).next = new_tail.clone() }; //1. Unsafe Method 3

                self.tail = new_tail;
            }
        }

        //    if self.tail.is_none() {
        //         self.tail = Some( Rc::new( RefCell::new( new_node ) ) );
        //    }
        //    else {
        //         let new_tail = Some( Rc::new( RefCell::new( new_node ) ) );
        //         self.tail.as_ref().unwrap().borrow_mut().next = new_tail.clone();
        //         self.tail = new_tail;
        //    }

        if self.head.is_none() {
            self.head = self.tail.clone();
        }
    }

    fn push_front(&mut self, data: T) {
        let new_node = Rc::new(RefCell::new(Node::new(data)));

        match self.head {
            None => self.head = Some(new_node),
            Some(_) => {
                let new_head = Some(new_node);
                new_head.as_ref().unwrap().borrow_mut().next = self.head.clone();
                self.head = new_head;
            }
        }

        if self.tail.is_none() {
            self.tail = self.head.clone();
        }
    }

    fn pop_front(&mut self) -> Option<T> {
        self.head.clone().map(|node| {
            let new_head = self.head.as_ref().unwrap().borrow().next.clone();
            self.head = new_head;

            if self.head.is_none() {
                self.tail = None;
            }

            let data = Rc::try_unwrap(node).ok().unwrap().into_inner().data; //Get the value of T
            data
        })
    }

    fn pop_back(&mut self) -> Option<T> {
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        self.tail.clone().map(|node| {
            let mut prev_node_to_tail_node = self.head.clone();

            while prev_node_to_tail_node.is_some() {
                let next = prev_node_to_tail_node
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .next
                    .clone();
                if next.as_ref().is_some()
                    && next.as_ref().unwrap().as_ptr() == self.tail.as_ref().unwrap().as_ptr()
                {
                    break;
                }

                prev_node_to_tail_node = next;
            }

            self.tail = prev_node_to_tail_node;

            match self.tail.as_ref() {
                None => self.head = None,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(tail) => unsafe { (*tail.as_ptr()).next = None }, //Maybe this unsafe code is not needed?
            }

            if self.tail.is_none() {
                self.head = None;
            }

            let data = Rc::try_unwrap(node).ok().unwrap().into_inner().data; //Get the value of T
            data
        })
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head.clone(),
            marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            current: self.head.clone(),
            marker: PhantomData,
        }
    }
}

impl<'a, T: Debug> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.current.is_none() {
            None
        } else {
            self.current.clone().map(|node| {
                // Need an unbound lifetime to get 'a
                let node = unsafe { &*node.as_ptr() }; //The unsafe is critical to allow to return the reference to &T
                self.current = node.next.clone();
                let data = &node.data;
                data
            })
        }
    }
}

impl<'a, T: Debug> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        if self.current.is_none() {
            None
        } else {
            self.current.clone().map(|node| {
                // Need an unbound lifetime to get 'a
                let node = unsafe { &mut *node.as_ptr() }; //The unsafe is critical to allow to return the reference to &mut T
                self.current = node.next.clone();
                let data = &mut node.data;
                data
            })
        }
    }
}

//#[cfg(test)]
mod test {

    use super::LinkedList;

    pub fn push_back_and_iter() {
        let mut linked_list = LinkedList::new();

        linked_list.push_back(10);
        linked_list.push_back(11);
        linked_list.push_back(12);
        linked_list.push_back(13);

        let mut iter = linked_list.iter();

        let mut item = iter.next();

        assert_eq!(item, Some(&10));

        item = iter.next();

        assert_eq!(item, Some(&11));

        item = iter.next();

        assert_eq!(item, Some(&12));

        item = iter.next();

        assert_eq!(item, Some(&13));

        item = iter.next();

        assert_eq!(item, None);

        item = iter.next();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_back_and_iter() {
        push_back_and_iter();
    }

    pub fn push_back_and_iter_mut() {
        let mut linked_list = LinkedList::new();

        linked_list.push_back(10);
        linked_list.push_back(11);
        linked_list.push_back(12);
        linked_list.push_back(13);

        let mut iter = linked_list.iter_mut();

        let mut item = iter.next();

        assert_eq!(item, Some(&mut 10));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 11));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 12));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 13));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, None);

        item = iter.next();

        assert_eq!(item, None);

        iter = linked_list.iter_mut();

        item = iter.next();

        assert_eq!(item, Some(&mut 20));

        item = iter.next();

        assert_eq!(item, Some(&mut 22));

        item = iter.next();

        assert_eq!(item, Some(&mut 24));

        item = iter.next();

        assert_eq!(item, Some(&mut 26));
    }

    #[test]
    fn test_push_back_and_iter_mut() {
        push_back_and_iter_mut();
    }

    pub fn push_front_and_iter() {
        let mut linked_list = LinkedList::new();

        linked_list.push_front(13);
        linked_list.push_front(12);
        linked_list.push_front(11);
        linked_list.push_front(10);

        let mut iter = linked_list.iter();

        let mut item = iter.next();

        assert_eq!(item, Some(&10));

        item = iter.next();

        assert_eq!(item, Some(&11));

        item = iter.next();

        assert_eq!(item, Some(&12));

        item = iter.next();

        assert_eq!(item, Some(&13));

        item = iter.next();

        assert_eq!(item, None);

        item = iter.next();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_front_and_iter() {
        push_front_and_iter();
    }

    pub fn push_front_and_iter_mut() {
        let mut linked_list = LinkedList::new();

        linked_list.push_front(13);
        linked_list.push_front(12);
        linked_list.push_front(11);
        linked_list.push_front(10);

        let mut iter = linked_list.iter_mut();

        let mut item = iter.next();

        assert_eq!(item, Some(&mut 10));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 11));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 12));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, Some(&mut 13));

        *item.unwrap() = &**item.as_ref().unwrap() * 2;

        item = iter.next();

        assert_eq!(item, None);

        item = iter.next();

        assert_eq!(item, None);

        iter = linked_list.iter_mut();

        item = iter.next();

        assert_eq!(item, Some(&mut 20));

        item = iter.next();

        assert_eq!(item, Some(&mut 22));

        item = iter.next();

        assert_eq!(item, Some(&mut 24));

        item = iter.next();

        assert_eq!(item, Some(&mut 26));

        item = iter.next();

        assert_eq!(item, None);

        item = iter.next();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_front_and_iter_mut() {
        push_front_and_iter_mut();
    }

    pub fn push_front_and_pop_front() {
        let mut linked_list = LinkedList::new();

        linked_list.push_front(13);
        linked_list.push_front(12);
        linked_list.push_front(11);
        linked_list.push_front(10);

        let mut item = linked_list.pop_front();

        assert_eq!(item, Some(10));

        item = linked_list.pop_front();

        assert_eq!(item, Some(11));

        item = linked_list.pop_front();

        assert_eq!(item, Some(12));

        item = linked_list.pop_front();

        assert_eq!(item, Some(13));

        item = linked_list.pop_front();

        assert_eq!(item, None);

        item = linked_list.pop_front();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_front_and_pop_front() {
        push_front_and_pop_front();
    }

    pub fn push_front_and_pop_back() {
        let mut linked_list = LinkedList::new();

        linked_list.push_front(13);
        linked_list.push_front(12);
        linked_list.push_front(11);
        linked_list.push_front(10);

        let mut item = linked_list.pop_back();

        assert_eq!(item, Some(13));

        item = linked_list.pop_back();

        assert_eq!(item, Some(12));

        item = linked_list.pop_back();

        assert_eq!(item, Some(11));

        item = linked_list.pop_back();

        assert_eq!(item, Some(10));

        item = linked_list.pop_back();

        assert_eq!(item, None);

        item = linked_list.pop_back();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_front_and_pop_back() {
        push_front_and_pop_back();
    }

    pub fn push_back_and_pop_back() {
        let mut linked_list = LinkedList::new();

        linked_list.push_back(10);
        linked_list.push_back(11);
        linked_list.push_back(12);
        linked_list.push_back(13);

        let mut item = linked_list.pop_back();

        assert_eq!(item, Some(13));

        item = linked_list.pop_back();

        assert_eq!(item, Some(12));

        item = linked_list.pop_back();

        assert_eq!(item, Some(11));

        item = linked_list.pop_back();

        assert_eq!(item, Some(10));

        item = linked_list.pop_back();

        assert_eq!(item, None);

        item = linked_list.pop_back();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_back_and_pop_back() {
        push_back_and_pop_back();
    }

    pub fn push_back_and_pop_front() {
        let mut linked_list = LinkedList::new();

        linked_list.push_back(10);
        linked_list.push_back(11);
        linked_list.push_back(12);
        linked_list.push_back(13);

        let mut item = linked_list.pop_front();

        assert_eq!(item, Some(10));

        item = linked_list.pop_front();

        assert_eq!(item, Some(11));

        item = linked_list.pop_front();

        assert_eq!(item, Some(12));

        item = linked_list.pop_front();

        assert_eq!(item, Some(13));

        item = linked_list.pop_front();

        assert_eq!(item, None);

        item = linked_list.pop_front();

        assert_eq!(item, None);
    }

    #[test]
    fn test_push_back_and_pop_front() {
        push_back_and_pop_front();
    }
}

fn main() {

    //use test;
    test::push_back_and_iter();
    test::push_back_and_iter_mut();
    test::push_back_and_pop_back();
    test::push_back_and_pop_front();
    test::push_front_and_iter();
    test::push_front_and_iter_mut();
    test::push_front_and_pop_back();
    test::push_front_and_pop_front();

    //test_push_front_and_pop_front();

    /*
    let mut linked_list = LinkedList::new();

    linked_list.push_back(10);
    linked_list.push_back(11);
    linked_list.push_back(12);
    linked_list.push_back(13);
    linked_list.push_front(9);
    linked_list.push_front(8);

    for item in linked_list.iter() {
        println!("Ref Value => {:?}", item);
    }

    for item in linked_list.iter_mut() {
        println!("RefMut Value => {:?}", item);
        *item = &*item * 2;
    }

    for item in linked_list.iter() {
        println!("Ref Value => {:?}", item);
    }

    let mut item = linked_list.pop_back();

    println!("pop_back Value => {:?}", item);

    item = linked_list.pop_back();

    println!("pop_back Value => {:?}", item);

    item = linked_list.pop_back();

    println!("pop_back Value => {:?}", item);

    item = linked_list.pop_back();

    println!("pop_back Value => {:?}", item);

    item = linked_list.pop_back();

    println!("pop_back Value => {:?}", item);

    item = linked_list.pop_front();

    println!("pop_front Value => {:?}", item);

    item = linked_list.pop_front();

    println!("pop_front Value => {:?}", item);

    for item in linked_list.iter() {
        println!("Ref Value => {:?}", item);
    }

    println!("Hello, world!");
    */
}
