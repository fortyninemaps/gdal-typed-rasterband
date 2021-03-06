pub mod typed_rasterband {
    use gdal::errors::Result as GdalResult;
    use gdal::raster::dataset::{Buffer, Dataset};
    use gdal::raster::rasterband::RasterBand;
    use gdal::raster::types::GdalType;
    use gdal_sys::GDALDataType;
    use std::error;
    use std::fmt;
    use std::marker::PhantomData;

    pub trait GdalFrom<T>: Sized {
        fn from(t: T) -> Self;
    }

    impl GdalFrom<f64> for u8 {
        fn from(d: f64) -> u8 {
            d as u8
        }
    }
    impl GdalFrom<f64> for u16 {
        fn from(d: f64) -> u16 {
            d as u16
        }
    }
    impl GdalFrom<f64> for u32 {
        fn from(d: f64) -> u32 {
            d as u32
        }
    }
    impl GdalFrom<f64> for i16 {
        fn from(d: f64) -> i16 {
            d as i16
        }
    }
    impl GdalFrom<f64> for i32 {
        fn from(d: f64) -> i32 {
            d as i32
        }
    }
    impl GdalFrom<f64> for f32 {
        fn from(d: f64) -> f32 {
            d as f32
        }
    }
    impl GdalFrom<f64> for f64 {
        fn from(d: f64) -> f64 {
            d as f64
        }
    }

    #[derive(Debug, Clone)]
    pub struct TypeError {}

    impl fmt::Display for TypeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "band type doesn't match type specified in caller")
        }
    }

    impl error::Error for TypeError {
        fn description(&self) -> &str {
            "band type doesn't match type specified in caller"
        }

        fn cause(&self) -> Option<&error::Error> {
            None
        }
    }

    pub struct TypedRasterBand<'a, T: Copy + GdalType> {
        rasterband: &'a RasterBand<'a>,
        pixel_type: PhantomData<&'a T>,
    }

    impl<'a, T: Copy + GdalType + GdalFrom<f64>> TypedRasterBand<'a, T> {
        pub fn from_rasterband(
            rasterband: &'a RasterBand,
        ) -> Result<TypedRasterBand<'a, T>, TypeError> {
            let pixel_type = PhantomData::<&'a T>;

            let bt = rasterband.band_type();

            if T::gdal_type() == bt {
                Ok(TypedRasterBand {
                    rasterband,
                    pixel_type,
                })
            } else {
                Err(TypeError {})
            }
        }

        pub fn owning_dataset(&self) -> &'a Dataset {
            self.rasterband.owning_dataset()
        }

        pub fn read(
            &self,
            window: (isize, isize),
            window_size: (usize, usize),
            size: (usize, usize),
        ) -> GdalResult<Buffer<T>> {
            self.rasterband.read_as(window, window_size, size)
        }

        pub fn read_band(&self) -> GdalResult<Buffer<T>> {
            self.rasterband.read_band_as()
        }

        pub fn write(
            &self,
            window: (isize, isize),
            window_size: (usize, usize),
            buffer: &Buffer<T>,
        ) -> GdalResult<()> {
            self.rasterband.write(window, window_size, buffer)
        }

        pub fn band_type(&self) -> GDALDataType::Type {
            self.rasterband.band_type()
        }

        pub fn no_data_value(&self) -> Option<T> {
            let no_data_f64 = self.rasterband.no_data_value();
            no_data_f64.map({ |f| T::from(f) })
        }

        pub fn scale(&self) -> Option<f64> {
            self.rasterband.scale()
        }

        pub fn offset(&self) -> Option<f64> {
            self.rasterband.offset()
        }
    }
}

#[cfg(test)]
mod tests {
    use gdal::raster::dataset::Dataset;
    use gdal_sys::GDALDataType;
    use std::path::Path;

    use super::typed_rasterband::*;

    #[test]
    fn typed_rasterband_u8() {
        let path = Path::new("testdata/test_u8.tif");
        let ds = Dataset::open(path).expect("failed to open test dataset");
        let band = ds.rasterband(1).expect("failed to read band");
        let typed_band = TypedRasterBand::<u8>::from_rasterband(&band).unwrap();

        assert_eq!(typed_band.band_type(), GDALDataType::GDT_Byte);
    }

    #[test]
    fn typed_rasterband_u16() {
        let path = Path::new("testdata/test_u16.tif");
        let ds = Dataset::open(path).expect("failed to open test dataset");
        let band = ds.rasterband(1).expect("failed to read band");
        let typed_band = TypedRasterBand::<u16>::from_rasterband(&band).unwrap();

        assert_eq!(typed_band.band_type(), GDALDataType::GDT_UInt16);
    }

    #[test]
    fn incorrect_type() {
        let path = Path::new("testdata/test_u16.tif");
        let ds = Dataset::open(path).expect("failed to open test dataset");
        let band = ds.rasterband(1).expect("failed to read band");
        let typed_band = TypedRasterBand::<u8>::from_rasterband(&band);

        assert!(typed_band.is_err());
    }

    #[test]
    fn typed_rasterband_u16_nodata() {
        let path = Path::new("testdata/test_u16_nodata.tif");
        let ds = Dataset::open(path).expect("failed to open test dataset");
        let band = ds.rasterband(1).expect("failed to read band");
        let typed_band = TypedRasterBand::<u16>::from_rasterband(&band).unwrap();

        assert_eq!(typed_band.no_data_value(), Some(42));
    }
}
