// Note:
// 10% are 0.1

pub struct Growth {
    yearly_growth: f32,
    inflation: Option<f32>,
}

impl Growth {
    pub fn new(yearly_growth: f32) -> Self {
        Self {
            yearly_growth: yearly_growth,
            inflation: None,
        }
    }

    pub fn new_with_inflation(yearly_growth: f32, inflation: f32) -> Self {
        Self {
            yearly_growth: yearly_growth,
            inflation: Some(inflation),
        }
    }

    pub fn get_nominal_growth(&self) -> f32 {
        self.yearly_growth
    }

    pub fn get_real_growth(self) -> f32 {
        match self.inflation {
            None => self.yearly_growth,
            Some(a) => (1. + self.yearly_growth) / (1. + a) - 1.,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_growth() {
        let x = Growth::new_with_inflation(0.1, 0.07);
        assert_eq!(x.get_real_growth(), 1.1 / 1.07 - 1.);
    }
}
