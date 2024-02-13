use crate::params::*;

#[derive(Copy, Clone)]
pub(crate) struct Location<S: Size> {
    bucket_index: S,
    inbucket_index: S,
}

impl<S: Size> Location<S> {
    pub fn bucket_index(&self) -> S {
        self.bucket_index
    }

    pub fn inbucket_index(&self) -> S {
        self.inbucket_index
    }
}

union TokenData<S: Size> {
    location: Location<S>,
    free_token_index: S,
}

pub(crate) struct Token<S: Size, U: UniqueTag> {
    tag: U,
    data: TokenData<S>,
}

impl<S: Size, U: UniqueTag> Token<S, U> {
    fn new(bucket_index: S, inbucket_index: S) -> Self {
        Self {
            tag: U::default(),
            data: TokenData {
                location: Location {
                    bucket_index,
                    inbucket_index,
                },
            },
        }
    }

    pub fn tag(&self) -> U {
        self.tag
    }

    pub unsafe fn location(&self) -> &Location<S> {
        debug_assert!(!self.tag.is_removed());
        debug_assert!(!self.tag.is_locked());
        &self.data.location
    }
}

pub(crate) struct TokenBucket<S: Size, U: UniqueTag> {
    tokens: Vec<Token<S, U>>,
    free_cursor: Option<S>,
}

impl<S: Size, U: UniqueTag> TokenBucket<S, U> {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            free_cursor: None,
        }
    }

    pub fn create(&mut self, bucket_index: S, inbucket_index: S) -> (S, U) {
        if let Some(free) = self.free_cursor {
            let usize_free = free.into();
            debug_assert!(usize_free < self.tokens.len());

            let token = unsafe { self.tokens.get_unchecked_mut(usize_free) };
            debug_assert!(token.tag.is_removed());
            debug_assert!(!token.tag.is_locked());

            let next_free = unsafe { token.data.free_token_index };
            self.free_cursor = if free != next_free {
                Some(next_free)
            } else {
                None
            };

            token.tag.set_removed(false);
            token.data.location = Location {
                bucket_index,
                inbucket_index,
            };
            return (free, token.tag);
        }
        let token_index = self.tokens.len();
        self.tokens.push(Token::new(bucket_index, inbucket_index));
        (token_index.into(), self.tokens.last().unwrap().tag)
    }

    pub fn mark_removed(&mut self, token_index: S) {
        let usize_token_index = token_index.into();
        debug_assert!(usize_token_index < self.tokens.len());

        let token = &mut self.tokens[usize_token_index];
        debug_assert!(!token.tag.is_removed());
        debug_assert!(!token.tag.is_locked());

        let tag = token.tag.next();
        if tag == token.tag {
            token.tag.mark_locked();
            return;
        }

        token.tag = tag;
        token.tag.set_removed(true);
        token.data.free_token_index = if let Some(free) = self.free_cursor {
            free
        } else {
            token_index
        };

        self.free_cursor = Some(token_index);
    }

    pub fn set_inbucket_index(&mut self, token_index: S, inbucket_index: S) {
        let usize_token_index = token_index.into();
        debug_assert!(usize_token_index < self.tokens.len());
        self.tokens[usize_token_index].data.location.inbucket_index = inbucket_index
    }

    pub fn try_get_token(&self, token_index: S) -> Option<&Token<S, U>> {
        let usize_token_index = token_index.into();
        if usize_token_index >= self.tokens.len() {
            return None;
        }
        Some(&self.tokens[usize_token_index])
    }

    pub fn contains(&self, token_index: S, tag: U) -> bool {
        match self.try_get_token(token_index) {
            Some(token) => tag == token.tag && !token.tag.is_removed() && !token.tag.is_locked(),
            None => false,
        }
    }

    pub fn reset_tokens(&mut self) {
        for token in self.tokens.iter_mut() {
            let removed = token.tag.is_removed();
            token.tag = U::default();
            token.tag.set_removed(removed);
        }
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    pub fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
    }
}

impl<S: Size, U: UniqueTag> Default for TokenBucket<S, U> {
    fn default() -> Self {
        Self {
            tokens: vec![],
            free_cursor: None,
        }
    }
}
