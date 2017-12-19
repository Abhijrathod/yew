use html::{VNode, Child, Listener};

#[macro_export]
macro_rules! html_impl {
    // Start of openging tag
    ($stack:ident (< $starttag:ident $($tail:tt)*)) => {
        let node = $crate::html::VNode::new(stringify!($starttag));
        $stack.push(node);
        html_impl! { $stack ($($tail)*) }
    };
    // PATTERN: class="",
    ($stack:ident (class = $class:expr, $($tail:tt)*)) => {
        $crate::macros::attach_class(&mut $stack, $class);
        html_impl! { $stack ($($tail)*) }
    };
    // PATTERN: value="",
    ($stack:ident (value = $value:expr, $($tail:tt)*)) => {
        $crate::macros::set_value(&mut $stack, $value);
        html_impl! { $stack ($($tail)*) }
    };
    // Events:
    ($stack:ident (onclick = $handler:expr, $($tail:tt)*)) => {
        html_impl! { $stack ((onclick) = $handler, $($tail)*) }
    };
    ($stack:ident (ondoubleclick = $handler:expr, $($tail:tt)*)) => {
        html_impl! { $stack ((ondoubleclick) = $handler, $($tail)*) }
    };
    ($stack:ident (onkeypress = $handler:expr, $($tail:tt)*)) => {
        html_impl! { $stack ((onkeypress) = $handler, $($tail)*) }
    };
    ($stack:ident (oninput = $handler:expr, $($tail:tt)*)) => {
        html_impl! { $stack ((oninput) = $handler, $($tail)*) }
    };
    // PATTERN: (action)=expression,
    ($stack:ident (($action:ident) = $handler:expr, $($tail:tt)*)) => {
        // Catch value to a separate variable for clear error messages
        let handler = $handler;
        let listener = $crate::html::$action::Wrapper::from(handler);
        $crate::macros::attach_listener(&mut $stack, Box::new(listener));
        html_impl! { $stack ($($tail)*) }
    };
    // PATTERN: attribute=value, - workaround for `type` attribute
    // because `type` is a keyword in Rust
    ($stack:ident (type = $val:expr, $($tail:tt)*)) => {
        $crate::macros::add_attribute(&mut $stack, "type", $val);
        html_impl! { $stack ($($tail)*) }
    };
    ($stack:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        $crate::macros::add_attribute(&mut $stack, stringify!($attr), $val);
        html_impl! { $stack ($($tail)*) }
    };
    // PATTERN: { for expression }
    ($stack:ident ({ for $eval:expr } $($tail:tt)*)) => {
        let nodes = $eval;
        for node in nodes.map($crate::html::Child::from) {
            $crate::macros::add_child(&mut $stack, node);
        }
        html_impl! { $stack ($($tail)*) }
    };
    // PATTERN: { expression }
    ($stack:ident ({ $eval:expr } $($tail:tt)*)) => {
        let node = $crate::html::Child::from($eval);
        $crate::macros::add_child(&mut $stack, node);
        html_impl! { $stack ($($tail)*) }
    };
    // End of openging tag
    ($stack:ident (> $($tail:tt)*)) => {
        html_impl! { $stack ($($tail)*) }
    };
    // Self-closing of tag
    ($stack:ident (/ > $($tail:tt)*)) => {
        $crate::macros::child_to_parent(&mut $stack, None);
        html_impl! { $stack ($($tail)*) }
    };
    // Traditional tag closing
    ($stack:ident (< / $endtag:ident > $($tail:tt)*)) => {
        let endtag = stringify!($endtag);
        $crate::macros::child_to_parent(&mut $stack, Some(endtag));
        html_impl! { $stack ($($tail)*) }
    };
    // "End of paring" rule
    ($stack:ident ()) => {
        $crate::macros::unpack($stack)
    };
}

// This entrypoint and implementation had separated to prevent infinite recursion.
#[macro_export]
macro_rules! html {
    ($($tail:tt)*) => {
        let mut stack = Vec::new();
        html_impl! { stack ($($tail)*) }
    };
}

type Stack<MSG> = Vec<VNode<MSG>>;

#[doc(hidden)]
pub fn unpack<MSG>(mut stack: Stack<MSG>) -> VNode<MSG> {
    if stack.len() != 1 {
        panic!("exactly one element have to be in html!");
    }
    stack.pop().unwrap()
}

#[doc(hidden)]
pub fn set_value<MSG, T: ToString>(stack: &mut Stack<MSG>, value: &T) {
    if let Some(node) = stack.last_mut() {
        node.set_value(value);
    } else {
        panic!("no tag to set value: {}", value.to_string());
    }
}

#[doc(hidden)]
pub fn add_attribute<MSG, T: ToString>(stack: &mut Stack<MSG>, name: &'static str, value: T) {
    if let Some(node) = stack.last_mut() {
        node.add_attribute(name, value);
    } else {
        panic!("no tag to set attribute: {}", name);
    }
}

#[doc(hidden)]
pub fn attach_class<MSG>(stack: &mut Stack<MSG>, class: &'static str) {
    if let Some(node) = stack.last_mut() {
        node.add_classes(class);
    } else {
        panic!("no tag to attach class: {}", class);
    }
}

#[doc(hidden)]
pub fn attach_listener<MSG>(stack: &mut Stack<MSG>, listener: Box<Listener<MSG>>) {
    if let Some(node) = stack.last_mut() {
        node.add_listener(listener);
    } else {
        panic!("no tag to attach listener: {:?}", listener);
    }
}

#[doc(hidden)]
pub fn add_child<MSG>(stack: &mut Stack<MSG>, child: Child<MSG>) {
    if let Some(parent) = stack.last_mut() {
        parent.add_child(child);
    } else {
        panic!("no nodes in stack to add child: {:?}", child);
    }
}

#[doc(hidden)]
pub fn child_to_parent<MSG>(stack: &mut Stack<MSG>, endtag: Option<&'static str>) {
    if let Some(node) = stack.pop() {
        let starttag = node.tag();
        if let Some(endtag) = endtag {
            if starttag != endtag {
                panic!("wrong closing tag: <{}> -> </{}>", starttag, endtag);
            }
        }
        if !stack.is_empty() {
            stack.last_mut().unwrap().add_child(Child::VNode(node));
        } else {
            // Keep the last node in the stack
            stack.push(node);
        }
    } else {
        panic!("redundant closing tag: {:?}", endtag);
    }
}