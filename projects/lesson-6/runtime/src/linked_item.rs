use support::{StorageMap, Parameter};
use sr_primitives::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct LinkedItem<Value> {
	pub prev: Option<Value>,
	pub next: Option<Value>,
}

pub struct LinkedList<Storage, Key, Value>(rstd::marker::PhantomData<(Storage, Key, Value)>);

impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where
    Value: Parameter + Member + Copy,
    Key: Parameter,
    Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query = Option<LinkedItem<Value>>>,
{
    fn read_head(key: &Key) -> LinkedItem<Value> {
 		Self::read(key, None)
 	}

  	fn write_head(account: &Key, item: LinkedItem<Value>) {
 		Self::write(account, None, item);
 	}

  	fn read(key: &Key, value: Option<Value>) -> LinkedItem<Value> {
 		Storage::get(&(key.clone(), value)).unwrap_or_else(|| LinkedItem {
 			prev: None,
 			next: None,
 		})
 	}

  	fn write(key: &Key, value: Option<Value>, item: LinkedItem<Value>) {
 		Storage::insert(&(key.clone(), value), item);
 	}

    pub fn append(key: &Key, value: Value) {
        // 作业：实现 append
		//首次添加：得到None，None
		//第二次添加：得到上次的kitty_id，上次的kitty_id（头kitty_id）
		
		let head = Self::read_head(key);
		let new_head = LinkedItem {
 			prev: Some(value),
 			next: head.next,//加入之前的链表头，首次添加：None
 		};

		//新加入的kitty成为链表头
		//链表头，最后一个，第一个
		Self::write_head(key, new_head);
		//首次添加：（Alice，None）-》{kitty_id, None}
		//第二次添加：（Alice，None）-》{本次kitty_id，上次kitty_id}

		//首次添加：得到new_head，{kitty_id, None}
		//第二次添加：得到(None,None),(上个，None)
		let prev = Self::read(key, head.prev);
		let new_prev = LinkedItem {
 			prev: prev.prev,//首次添加，为kitty_id。第二次添加，上一个
 			next: Some(value),
 		};
		//首次添加：把头替换成（Alice，None）-》{kitty_id, kitty_id}
		//第二次添加：(Alice,上次kitty_id) -》{None,本次kitty_id}
		//链表中：上一个，下一个
		Self::write(key, head.prev, new_prev);

		//首次添加，None，None
		//第二次添加，上次的id，None
		let item = LinkedItem {
 			prev: head.prev,
 			next: None,
 		};

		//首次添加：把头替换成（Alice，kitty_id）-》{None, None}
		//第二次添加：（Alice，kitty_id）-》{上次kitty_id, None}
		//链表尾：上一个，None
 		Self::write(key, Some(value), item);
    }

    pub fn remove(key: &Key, value: Value) {
        // 作业：实现 remove
		if let Some(item) = Storage::get(&(key.clone(), Some(value))) {
			let prev = Self::read(key, item.prev);
			let new_prev = LinkedItem {
 				prev: prev.prev,
 				next: item.next,
 			};

			Self::write(key, item.prev, new_prev);

			let next = Self::read(key, item.next);
 			let new_next = LinkedItem {
 				prev: item.prev,
 				next: next.next,
 			};

  			Self::write(key, item.next, new_next);
			
			//添加：删除Storage
			Storage::remove(&(key.clone(), Some(value)));
		}
    }
}