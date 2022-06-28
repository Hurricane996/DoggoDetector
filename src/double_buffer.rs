pub struct DoubleBuffer<T> {
    a: T,
    b: T,
    front: FrontBuffer
}

enum FrontBuffer {
    A,
    B
}

impl<T> DoubleBuffer<T> {
    pub fn swap(&mut self) {
        self.front = match self.front {
            FrontBuffer::A => FrontBuffer::B,
            FrontBuffer::B => FrontBuffer::A,
        }
    }

    pub fn buffers(&mut self) -> (&T, &mut T) {
        match self.front {
            FrontBuffer::A => (&self.a, &mut self.b),
            FrontBuffer::B => (&self.b, &mut self.a),
        }
    }

    pub fn front(&self) -> &T {
        match self.front {
            FrontBuffer::A => &self.a,
            FrontBuffer::B => &self.b,
        }
    }

    pub fn back(&mut self) -> &mut T {
        match self.front {
            FrontBuffer::A => &mut self.b,
            FrontBuffer::B => &mut self.a,
        }
    }




    pub fn to_front(self) -> T {
        match self.front {
            FrontBuffer::A => self.a,
            FrontBuffer::B => self.b
        }
    }
}

impl <T> DoubleBuffer<T> where T: Clone {
    pub fn clone_front(&self) -> T {
        match self.front {
            FrontBuffer::A => self.a.clone(),
            FrontBuffer::B => self.b.clone()
        }
    }
}

impl <T> Default for DoubleBuffer<T> where T: Default {
    fn default() -> Self {
        Self { a: Default::default(), b: Default::default(), front: FrontBuffer::A }
    }
}