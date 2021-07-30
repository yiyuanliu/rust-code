use rand::random;

struct ListNode {
    elem: i32,
    next: Option<Box<ListNode>>,
}

fn qsort(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if head.is_none() {
        return None;
    }

    let (mut l1, mut l2) = (None, None);
    let (mut t1, mut t2) = (&mut l1, &mut l2);
    let mut head = head;
    let mut curr = head.as_mut().unwrap().next.take();
    while let Some(mut node) = curr {
        curr = node.next.take();
        if node.elem < head.as_ref().unwrap().elem {
            // add to l1
            *t1 = Some(node);
            t1 = &mut t1.as_mut().unwrap().next;
        } else {
            // add to t2
            *t2 = Some(node);
            t2 = &mut t2.as_mut().unwrap().next;
        }
    }

    let mut l1 = qsort(l1);
    let l2 = qsort(l2);

    // l1, head, l2
    head.as_mut().unwrap().next = l2;
    let mut curr = l1.as_mut();
    while let Some(mut node) = curr {
        if node.next.is_none() {
            node.next = head;
            return l1;
        }
        curr = node.next.as_mut();
    }

    head
}

fn gen(len: usize) -> Option<Box<ListNode>> {
    let mut head = None;
    let mut tail = &mut head;
    for _ in 0..len {
        let val = random();
        *tail = Some(Box::new(ListNode { elem: val, next: None } ));
        tail = &mut tail.as_mut().unwrap().next;
    }

    head
}

fn is_sorted(head: &Option<Box<ListNode>>) -> bool {
    let mut curr = head.as_ref();
    let mut prev = i32::MIN;
    while let Some(node) = curr {
        if node.elem < prev {
            return false;
        }
        prev = node.elem;
        curr = node.next.as_ref();
    }

    return true;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        for _ in 0..10 {
            let len = random::<usize>() % 1000;
            let list = gen(len);
            let sorted = qsort(list);
            assert_eq!(is_sorted(&sorted), true);
        }
    }
}