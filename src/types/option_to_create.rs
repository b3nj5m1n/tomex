use std::fmt::Display;

pub enum OptionToCreate<T>
where
    T: Display,
{
    Value(T),
    Create,
}

impl<T> OptionToCreate<T>
where
    T: Sized,
    T: Display,
{
    /// Takes a vector of T and turns it into a vector of OptionToCreate<T> with an additional
    /// OptionToCreate::Create at the beginning of the vector.
    pub fn create_option_to_create(v: Vec<T>) -> Vec<OptionToCreate<T>> {
        let mut options = vec![OptionToCreate::Create];
        options.append(
            &mut v
                .into_iter()
                .map(|x| OptionToCreate::Value(x))
                .collect::<Vec<_>>(),
        );
        options
    }
}

impl<T> Display for OptionToCreate<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionToCreate::Value(value) => write!(f, "{}", value),
            OptionToCreate::Create => write!(f, "Create new"),
        }
    }
}
