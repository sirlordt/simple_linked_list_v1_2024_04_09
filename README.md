This project is ilustrative for iter() and iter_mut().

## Linked list And Node Definitio

```rust

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

impl<T: Debug> LinkedList<T> {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    //... Other functions here ...

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
```

## Struct Definition

```rust
pub struct Iter<'a, T: Debug + 'a> {
    current: Link<T>,
    marker: PhantomData<&'a Node<T>>, //A trick needed to allow return &T. See the Iter.next() method
}

pub struct IterMut<'a, T: Debug + 'a> {
    current: Link<T>,
    marker: PhantomData<&'a mut Node<T>>, //A trick needed to allow return &mut T. See the ItemMut.next() method
}
```

## Struct Implementation

```rust
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
```
