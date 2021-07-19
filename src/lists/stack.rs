use std::rc::Rc as Rc;

struct List<T> {
    head: Option<Rc<Node<T>>>,
}

struct Node<T> {
    val: T,
    next: Option<Rc<Node<T>>>,
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<T> List<T> {

    fn new() -> List<T> {
        List { head: None }
    }

    fn pop(&self) -> List<T> {
        List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
    }

    fn push(&self, val: T) -> List<T> {
        List { head: Some(Rc::new(Node {
            val: val,
            next: self.head.clone(),
        }))}
    }

    fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.val)
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn test() {
        let list = List::new();
        assert_eq!(list.peek(), None);
        
        let list = list.push(10);
        assert_eq!(list.peek(), Some(&10));

        let list = list.push(100);
        assert_eq!(list.peek(), Some(&100));

        let list = list.pop();
        assert_eq!(list.peek(), Some(&10));
    }
}