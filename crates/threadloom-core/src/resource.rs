use crate::signal::{ReadSignal, WriteSignal, create_effect, create_signal};
pub struct Action<I, O> {
    is_loading: ReadSignal<bool>,
    set_loading: WriteSignal<bool>,
    func: std::rc::Rc<dyn Fn(I) -> std::pin::Pin<Box<dyn std::future::Future<Output = O>>>>,
}

impl<I: 'static, O: 'static> Clone for Action<I, O> {
    fn clone(&self) -> Self {
        Self {
            is_loading: self.is_loading,
            set_loading: self.set_loading,
            func: self.func.clone(),
        }
    }
}

impl<I: 'static, O: 'static> Action<I, O> {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(I) -> Fut + 'static,
        Fut: std::future::Future<Output = O> + 'static,
    {
        let (is_loading, set_loading) = create_signal(false);
        let func = std::rc::Rc::new(move |i| {
            Box::pin(f(i)) as std::pin::Pin<Box<dyn std::future::Future<Output = O>>>
        });
        Self {
            is_loading,
            set_loading,
            func,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading.get()
    }

    pub async fn execute(&self, input: I) -> O {
        self.set_loading.set(true);
        let res = (self.func)(input).await;
        self.set_loading.set(false);
        res
    }
}

pub struct Resource<T> {
    data: ReadSignal<Option<T>>,
    loading: ReadSignal<bool>,
    error: ReadSignal<Option<String>>,
    refetch: WriteSignal<u32>,
}

impl<T: Clone + 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            loading: self.loading,
            error: self.error,
            refetch: self.refetch,
        }
    }
}

impl<T: Clone + 'static> Resource<T> {
    pub fn get(&self) -> Option<T> {
        self.data.get()
    }

    pub fn is_loading(&self) -> bool {
        self.loading.get()
    }

    pub fn error(&self) -> Option<String> {
        self.error.get()
    }

    pub fn refetch(&self) {
        self.refetch.update(|v| *v += 1);
    }
}

pub fn create_resource<S, T, F, Fut>(source: impl Fn() -> S + 'static, fetcher: F) -> Resource<T>
where
    S: Clone + PartialEq + 'static,
    T: Clone + PartialEq + 'static,
    F: Fn(S) -> Fut + 'static,
    Fut: std::future::Future<Output = Result<T, String>> + 'static,
{
    let (data, set_data) = create_signal(None);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None);
    let (refetch_sig, set_refetch) = create_signal(0u32);

    let fetcher = std::rc::Rc::new(fetcher);
    let source_rc = std::rc::Rc::new(source);

    create_effect(move || {
        let source_val = source_rc();
        let _ = refetch_sig.get();

        set_loading.set(true);
        set_error.set(None);

        let fut = fetcher(source_val);

        let set_data_clone = set_data.clone();
        let set_loading_clone = set_loading.clone();
        let set_error_clone = set_error.clone();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                match fut.await {
                    Ok(val) => {
                        set_data_clone.set(Some(val));
                    }
                    Err(e) => {
                        set_error_clone.set(Some(e));
                    }
                }
                set_loading_clone.set(false);
            });
        }
    });

    Resource {
        data,
        loading,
        error,
        refetch: set_refetch,
    }
}
