//! Axis Aligned Bounding Boxes.

use nalgebra::{Point3, Vector3};
use std::f32;
use std::ops::Index;
use std::fmt::{Display, Formatter, Result};
use axis::Axis;
use ray::Ray;
use raycast::{Intersectable, RaycastResult};

/// AABB struct.
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    /// Minimum coordinates
    pub min: Point3<f32>,

    /// Maximum coordinates
    pub max: Point3<f32>,
}

/// A trait implemented by things which can be bounded by an [`AABB`].
///
/// [`AABB`]: struct.AABB.html
///
pub trait Bounded {
    /// Returns the geometric bounds of this object in the form of an [`AABB`].
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::{AABB, Bounded};
    /// use bvh::nalgebra::Point3;
    ///
    /// struct Something;
    ///
    /// impl Bounded for Something {
    ///     fn aabb(&self) -> AABB {
    ///         let point1 = Point3::new(0.0,0.0,0.0);
    ///         let point2 = Point3::new(1.0,1.0,1.0);
    ///         AABB::with_bounds(point1, point2)
    ///     }
    /// }
    ///
    /// let something = Something;
    /// let aabb = something.aabb();
    ///
    /// assert!(aabb.contains(&Point3::new(0.0,0.0,0.0)));
    /// assert!(aabb.contains(&Point3::new(1.0,1.0,1.0)));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    fn aabb(&self) -> AABB;
}

impl AABB {
    /// Creates a new [`AABB`] with the given bounds.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let aabb = AABB::with_bounds(Point3::new(-1.0,-1.0,-1.0), Point3::new(1.0,1.0,1.0));
    /// assert_eq!(aabb.min.x, -1.0);
    /// assert_eq!(aabb.max.z, 1.0);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn with_bounds(min: Point3<f32>, max: Point3<f32>) -> AABB {
        AABB {
            min: min,
            max: max,
        }
    }

    /// Creates a new empty [`AABB`].
    ///
    /// # Examples
    /// ```
    /// # extern crate bvh;
    /// # extern crate rand;
    /// use bvh::aabb::AABB;
    ///
    /// # fn main() {
    /// let aabb = AABB::empty();
    /// let min = &aabb.min;
    /// let max = &aabb.max;
    ///
    /// // For any point
    /// let x = rand::random();
    /// let y = rand::random();
    /// let z = rand::random();
    ///
    /// // An empty AABB should not contain it
    /// assert!(x < min.x && y < min.y && z < min.z);
    /// assert!(max.x < x && max.y < y && max.z < z);
    /// # }
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn empty() -> AABB {
        AABB {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// Returns true if the [`Point3`] is inside the [`AABB`].
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let aabb = AABB::with_bounds(Point3::new(-1.0,-1.0,-1.0), Point3::new(1.0,1.0,1.0));
    /// let point_inside = Point3::new(0.125,-0.25,0.5);
    /// let point_outside = Point3::new(1.0,-2.0,4.0);
    ///
    /// assert!(aabb.contains(&point_inside));
    /// assert!(!aabb.contains(&point_outside));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    /// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
    ///
    pub fn contains(&self, p: &Point3<f32>) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y &&
        p.z >= self.min.z && p.z <= self.max.z
    }

    /// Returns true if the [`Point3`] is approximately inside the [`AABB`]
    /// with respect to some `epsilon`.
    ///
    /// # Examples
    /// ```
    /// use bvh::EPSILON;
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let aabb = AABB::with_bounds(Point3::new(-1.0,-1.0,-1.0), Point3::new(1.0,1.0,1.0));
    /// let point_barely_inside = Point3::new(1.0000001,-1.0000001,1.000000001);
    /// let point_outside = Point3::new(1.0,-2.0,4.0);
    ///
    /// assert!(aabb.approx_contains_eps(&point_barely_inside, EPSILON));
    /// assert!(!aabb.approx_contains_eps(&point_outside, EPSILON));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    /// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
    ///
    pub fn approx_contains_eps(&self, p: &Point3<f32>, epsilon: f32) -> bool {
        (p.x - self.min.x) > -epsilon && (p.x - self.max.x) < epsilon &&
        (p.y - self.min.y) > -epsilon && (p.y - self.max.y) < epsilon &&
        (p.z - self.min.z) > -epsilon && (p.z - self.max.z) < epsilon
    }

    /// Returns a new minimal [`AABB`] which contains both this [`AABB`] and `other`.
    /// The result is the convex hull of the both [`AABB`]s.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let aabb1 = AABB::with_bounds(Point3::new(-101.0,0.0,0.0), Point3::new(-100.0,1.0,1.0));
    /// let aabb2 = AABB::with_bounds(Point3::new(100.0,0.0,0.0), Point3::new(101.0,1.0,1.0));
    /// let joint = aabb1.join(&aabb2);
    ///
    /// let point_inside_aabb1 = Point3::new(-100.5,0.5,0.5);
    /// let point_inside_aabb2 = Point3::new(100.5,0.5,0.5);
    /// let point_inside_joint = Point3::new(0.0,0.5,0.5);
    ///
    /// # assert!(aabb1.contains(&point_inside_aabb1));
    /// # assert!(!aabb1.contains(&point_inside_aabb2));
    /// # assert!(!aabb1.contains(&point_inside_joint));
    /// #
    /// # assert!(!aabb2.contains(&point_inside_aabb1));
    /// # assert!(aabb2.contains(&point_inside_aabb2));
    /// # assert!(!aabb2.contains(&point_inside_joint));
    ///
    /// assert!(joint.contains(&point_inside_aabb1));
    /// assert!(joint.contains(&point_inside_aabb2));
    /// assert!(joint.contains(&point_inside_joint));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn join(&self, other: &AABB) -> AABB {
        AABB::with_bounds(Point3::new(self.min.x.min(other.min.x),
                                      self.min.y.min(other.min.y),
                                      self.min.z.min(other.min.z)),
                          Point3::new(self.max.x.max(other.max.x),
                                      self.max.y.max(other.max.y),
                                      self.max.z.max(other.max.z)))
    }

    /// Returns a new minimal [`AABB`] which contains both
    /// this [`AABB`] and the [`Point3`] `other`.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let point1 = Point3::new(0.0,0.0,0.0);
    /// let point2 = Point3::new(1.0,1.0,1.0);
    /// let point3 = Point3::new(2.0,2.0,2.0);
    ///
    /// let aabb = AABB::empty();
    /// assert!(!aabb.contains(&point1));
    ///
    /// let aabb1 = aabb.grow(&point1);
    /// assert!(aabb1.contains(&point1));
    ///
    /// let aabb2 = aabb.grow(&point2);
    /// assert!(aabb2.contains(&point2));
    /// assert!(!aabb2.contains(&point3));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    /// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
    ///
    pub fn grow(&self, other: &Point3<f32>) -> AABB {
        AABB::with_bounds(Point3::new(self.min.x.min(other.x),
                                      self.min.y.min(other.y),
                                      self.min.z.min(other.z)),
                          Point3::new(self.max.x.max(other.x),
                                      self.max.y.max(other.y),
                                      self.max.z.max(other.z)))
    }

    /// Returns a new minimal [`AABB`] which contains both this [`AABB`] and the [`Bounded`] `other`.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::{AABB, Bounded};
    /// use bvh::nalgebra::Point3;
    ///
    /// struct Something;
    ///
    /// impl Bounded for Something {
    ///     fn aabb(&self) -> AABB {
    ///         let point1 = Point3::new(0.0,0.0,0.0);
    ///         let point2 = Point3::new(1.0,1.0,1.0);
    ///         AABB::with_bounds(point1, point2)
    ///     }
    /// }
    ///
    /// let aabb = AABB::empty();
    /// let something = Something;
    /// let aabb1 = aabb.join_bounded(&something);
    ///
    /// let center = something.aabb().center();
    /// assert!(aabb1.contains(&center));
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    /// [`Bounded`]: trait.Bounded.html
    ///
    pub fn join_bounded<T: Bounded>(&self, other: &T) -> AABB {
        self.join(&other.aabb())
    }

    /// Returns the size of this [`AABB`] in all three dimensions.
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let aabb = AABB::with_bounds(Point3::new(-1.0,-1.0,-1.0), Point3::new(1.0,1.0,1.0));
    /// let size = aabb.size();
    /// assert!(size.x == 2.0 && size.y == 2.0 && size.z == 2.0);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    /// Returns the center [`Point3`] of the [`AABB`].
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let min = Point3::new(41.0,41.0,41.0);
    /// let max = Point3::new(43.0,43.0,43.0);
    ///
    /// let aabb = AABB::with_bounds(min, max);
    /// let center = aabb.center();
    /// assert!(center.x == 42.0 && center.y == 42.0 && center.z == 42.0);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    /// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
    ///
    pub fn center(&self) -> Point3<f32> {
        self.min + (self.size() / 2.0)
    }

    /// Returns the total surface area of this [`AABB`].
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let min = Point3::new(41.0,41.0,41.0);
    /// let max = Point3::new(43.0,43.0,43.0);
    ///
    /// let aabb = AABB::with_bounds(min, max);
    /// let surface_area = aabb.surface_area();
    /// assert!(surface_area == 24.0);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.x * size.z + size.y * size.z)
    }

    /// Returns the volume of this [`AABB`].
    ///
    /// # Examples
    /// ```
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let min = Point3::new(41.0,41.0,41.0);
    /// let max = Point3::new(43.0,43.0,43.0);
    ///
    /// let aabb = AABB::with_bounds(min, max);
    /// let volume = aabb.volume();
    /// assert!(volume == 8.0);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Returns the axis along which the [`AABB`] is stretched the most.
    ///
    /// # Examples
    /// ```
    /// use bvh::axis::Axis;
    /// use bvh::aabb::AABB;
    /// use bvh::nalgebra::Point3;
    ///
    /// let min = Point3::new(-100.0,0.0,0.0);
    /// let max = Point3::new(100.0,0.0,0.0);
    ///
    /// let aabb = AABB::with_bounds(min, max);
    /// let axis = aabb.largest_axis();
    /// assert!(axis == Axis::X);
    /// ```
    ///
    /// [`AABB`]: struct.AABB.html
    ///
    pub fn largest_axis(&self) -> Axis {
        let size = self.size();
        if size.x > size.y && size.x > size.z {
            Axis::X
        } else if size.y > size.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }
}

/// Default instance for [`AABB`]s. Returns an [`AABB`] which is [`empty()`].
///
/// [`AABB`]: struct.AABB.html
/// [`empty()`]: #method.empty
///
impl Default for AABB {
    fn default() -> AABB {
        AABB::empty()
    }
}

/// Make [`AABB`]s indexable. `aabb[0]` gives a reference to the minimum bound.
/// All other indices return a reference to the maximum bound.
///
/// # Examples
/// ```
/// use bvh::aabb::AABB;
/// use bvh::nalgebra::Point3;
///
/// let min = Point3::new(3.0,4.0,5.0);
/// let max = Point3::new(123.0,123.0,123.0);
///
/// let aabb = AABB::with_bounds(min, max);
/// assert_eq!(aabb[0], min);
/// assert_eq!(aabb[1], max);
/// ```
///
/// [`AABB`]: struct.AABB.html
///
impl Index<usize> for AABB {
    type Output = Point3<f32>;

    fn index(&self, index: usize) -> &Point3<f32> {
        if index == 0 { &self.min } else { &self.max }
    }
}

/// Implementation of [`Bounded`] for [`AABB`].
///
/// # Examples
/// ```
/// use bvh::aabb::{AABB, Bounded};
/// use bvh::nalgebra::Point3;
///
/// let point_a = Point3::new(3.0,4.0,5.0);
/// let point_b = Point3::new(17.0,18.0,19.0);
/// let aabb = AABB::empty().grow(&point_a).grow(&point_b);
///
/// let aabb_aabb = aabb.aabb();
///
/// assert_eq!(aabb_aabb.min, aabb.min);
/// assert_eq!(aabb_aabb.max, aabb.max);
/// ```
///
/// [`AABB`]: struct.AABB.html
/// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
///
impl Bounded for AABB {
    fn aabb(&self) -> AABB {
        *self
    }
}

/// Implementation of [`Bounded`] for [`Point3`].
///
/// # Examples
/// ```
/// use bvh::aabb::{AABB, Bounded};
/// use bvh::nalgebra::Point3;
///
/// let point = Point3::new(3.0,4.0,5.0);
///
/// let aabb = point.aabb();
/// assert!(aabb.contains(&point));
/// ```
///
/// [`Bounded`]: trait.Bounded.html
/// [`Point3`]: http://nalgebra.org/doc/nalgebra/struct.Point3.html
///
impl Bounded for Point3<f32> {
    fn aabb(&self) -> AABB {
        AABB::with_bounds(*self, *self)
    }
}

/// Intersections between [`Ray`]s and [`AABB`]s.
///
/// [`AABB`]: struct.AABB.html
/// [`Ray`]: struct.Ray.html
///
impl Intersectable for AABB {
    /// Tests the intersection of a [`Ray`] with an [`AABB`] using the optimized algorithm
    /// from [this paper](http://www.cs.utah.edu/~awilliam/box/box.pdf).
    fn does_intersect(&self, ray: &Ray) -> bool {
        let mut ray_min = (self[ray.sign().x].x - ray.origin.x) * ray.inv_direction().x;
        let mut ray_max = (self[1 - ray.sign().x].x - ray.origin.x) * ray.inv_direction().x;

        let y_min = (self[ray.sign().y].y - ray.origin.y) * ray.inv_direction().y;
        let y_max = (self[1 - ray.sign().y].y - ray.origin.y) * ray.inv_direction().y;

        if (ray_min > y_max) || (y_min > ray_max) {
            return false;
        }

        if y_min > ray_min {
            ray_min = y_min;
        }
        // Using the following solution significantly decreases the performance
        // ray_min = ray_min.max(y_min);

        if y_max < ray_max {
            ray_max = y_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        let z_min = (self[ray.sign().z].z - ray.origin.z) * ray.inv_direction().z;
        let z_max = (self[1 - ray.sign().z].z - ray.origin.z) * ray.inv_direction().z;

        if (ray_min > z_max) || (z_min > ray_max) {
            return false;
        }

        // Only required for bounded intersection intervals.
        // if z_min > ray_min {
        // ray_min = z_min;
        // }

        if z_max < ray_max {
            ray_max = z_max;
        }
        // Using the following solution significantly decreases the performance
        // ray_max = ray_max.min(y_max);

        ray_max > 0.0
    }

    fn intersection(&self, ray: &Ray) -> RaycastResult {
        let hit_min_x = (self.min.x - ray.origin.x) * ray.inv_direction().x;
        let hit_max_x = (self.max.x - ray.origin.x) * ray.inv_direction().x;

        let hit_min_y = (self.min.y - ray.origin.y) * ray.inv_direction().y;
        let hit_max_y = (self.max.y - ray.origin.y) * ray.inv_direction().y;

        let hit_min_z = (self.min.z - ray.origin.z) * ray.inv_direction().z;
        let hit_max_z = (self.max.z - ray.origin.z) * ray.inv_direction().z;

        let x_entry = hit_min_x.min(hit_max_x);
        let y_entry = hit_min_y.min(hit_max_y);
        let z_entry = hit_min_z.min(hit_max_z);
        let x_exit = hit_min_x.max(hit_max_x);
        let y_exit = hit_min_y.max(hit_max_y);
        let z_exit = hit_min_z.max(hit_max_z);

        let latest_entry = x_entry.max(y_entry).max(z_entry);
        let earliest_exit = x_exit.min(y_exit).min(z_exit);

        if latest_entry < earliest_exit && earliest_exit > 0.0 {
            RaycastResult::hit(latest_entry)
        } else {
            RaycastResult::Miss
        }
    }
}

/// TODO comment
///
impl Display for AABB {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f,
               "(X: {} <> {}, Y: {} <> {}, Z: {} <> {})",
               self.min.x,
               self.max.x,
               self.min.y,
               self.max.y,
               self.min.z,
               self.max.z,)
    }
}

#[cfg(test)]
mod tests {
    use EPSILON;
    use aabb::{AABB, Bounded};
    use nalgebra::{Point3, Vector3};

    type TupleVec = (f32, f32, f32);

    fn to_point(tpl: &TupleVec) -> Point3<f32> {
        Point3::new(tpl.0, tpl.1, tpl.2)
    }

    fn to_vector(tpl: &TupleVec) -> Vector3<f32> {
        Vector3::new(tpl.0, tpl.1, tpl.2)
    }

    /// Test whether an empty `AABB` does not contains anything.
    quickcheck!{
        fn test_empty_contains_nothing(tpl: TupleVec) -> bool {
            // Define a random Point
            let p = to_point(&tpl);

            // Create an empty AABB
            let aabb = AABB::empty();

            // It should not contain anything
            !aabb.contains(&p)
        }
    }

    /// Test whether a default `AABB` is empty.
    quickcheck!{
        fn test_default_is_empty(tpl: TupleVec) -> bool {
            // Define a random Point
            let p = to_point(&tpl);

            // Create a default AABB
            let aabb: AABB = Default::default();

            // It should not contain anything
            !aabb.contains(&p)
        }
    }

    /// Test whether an `AABB` always contains its center.
    quickcheck!{
        fn test_aabb_contains_center(a: TupleVec, b: TupleVec) -> bool {
            // Define two points which will be the corners of the `AABB`
            let p1 = to_point(&a);
            let p2 = to_point(&b);

            // Span the `AABB`
            let aabb = AABB::empty().grow(&p1).join_bounded(&p2);

            // Its center should be inside the `AABB`
            aabb.contains(&aabb.center())
        }
    }

    /// Test whether the joint of two point-sets contains all the points.
    quickcheck!{
        fn test_join_two_aabbs(a: (TupleVec, TupleVec, TupleVec, TupleVec, TupleVec),
                               b: (TupleVec, TupleVec, TupleVec, TupleVec, TupleVec))
                               -> bool {
            // Define an array of ten points
            let points = [a.0, a.1, a.2, a.3, a.4, b.0, b.1, b.2, b.3, b.4];

            // Convert these points to `Point3`
            let points = points.iter().map(to_point).collect::<Vec<Point3<f32>>>();

            // Create two `AABB`s. One spanned the first five points,
            // the other by the last five points
            let aabb1 = points.iter().take(5).fold(AABB::empty(), |aabb, point| aabb.grow(&point));
            let aabb2 = points.iter().skip(5).fold(AABB::empty(), |aabb, point| aabb.grow(&point));

            // The `AABB`s should contain the points by which they are spanned
            let aabb1_contains_init_five = points.iter()
                .take(5)
                .fold(true, |b, point| b && aabb1.contains(&point));
            let aabb2_contains_last_five = points.iter()
                .skip(5)
                .fold(true, |b, point| b && aabb2.contains(&point));

            // Build the joint of the two `AABB`s
            let aabbu = aabb1.join(&aabb2);

            // The joint should contain all points
            let aabbu_contains_all = points.iter()
                .fold(true, |b, point| b && aabbu.contains(&point));

            // Return the three properties
            aabb1_contains_init_five && aabb2_contains_last_five && aabbu_contains_all
        }
    }

    /// Test whether some points relative to the center of an AABB are classified correctly.
    quickcheck!{
        fn test_points_relative_to_center_and_size(a: TupleVec, b: TupleVec) -> bool {
            // Generate some nonempty AABB
            let aabb = AABB::empty()
                .grow(&to_point(&a))
                .grow(&to_point(&b));

            // Get its size and center
            let size = aabb.size();
            let size_half = size / 2.0;
            let center = aabb.center();

            // Compute the min and the max corners of the AABB by hand
            let inside_ppp = center + size_half;
            let inside_mmm = center - size_half;

            // Generate two points which are outside the AABB
            let outside_ppp = inside_ppp + Vector3::new(0.1, 0.1, 0.1);
            let outside_mmm = inside_mmm - Vector3::new(0.1, 0.1, 0.1);

            assert!(aabb.approx_contains_eps(&inside_ppp, EPSILON));
            assert!(aabb.approx_contains_eps(&inside_mmm, EPSILON));
            assert!(!aabb.contains(&outside_ppp));
            assert!(!aabb.contains(&outside_mmm));

            true
        }
    }

    /// Test whether the surface of a nonempty AABB is always positive.
    quickcheck!{
        fn test_surface_always_positive(a: TupleVec, b: TupleVec) -> bool {
            let aabb = AABB::empty()
                .grow(&to_point(&a))
                .grow(&to_point(&b));
            aabb.surface_area() >= 0.0
        }
    }

    /// Compute and compare the surface area of an AABB by hand.
    quickcheck!{
        fn test_surface_area_cube(pos: TupleVec, size: f32) -> bool {
            // Generate some non-empty AABB
            let pos = to_point(&pos);
            let size_vec = Vector3::new(size, size, size);
            let aabb = AABB::with_bounds(pos, pos + size_vec);

            // Check its surface area
            let area_a = aabb.surface_area();
            let area_b = 6.0 * size * size;
            (1.0 - (area_a / area_b)).abs() < EPSILON
        }
    }

    /// Test whether the volume of a nonempty AABB is always positive.
    quickcheck!{
        fn test_volume_always_positive(a: TupleVec, b: TupleVec) -> bool {
            let aabb = AABB::empty()
                .grow(&to_point(&a))
                .grow(&to_point(&b));
            aabb.volume() >= 0.0
        }
    }

    /// Compute and compare the volume of an AABB by hand.
    quickcheck!{
        fn test_volume_by_hand(pos: TupleVec, size: TupleVec) -> bool {
            // Generate some non-empty AABB
            let pos = to_point(&pos);
            let size = to_vector(&size);
            let aabb = pos.aabb().grow(&(pos + size));

            // Check its volume
            let volume_a = aabb.volume();
            let volume_b = (size.x * size.y * size.z).abs();
            (1.0 - (volume_a / volume_b)).abs() < EPSILON
        }
    }

    /// Test whether generating an `AABB` from the min and max bounds yields the same `AABB`.
    quickcheck!{
        fn test_create_aabb_from_indexable(a: TupleVec, b: TupleVec, p: TupleVec) -> bool {
            // Create a random point
            let point = to_point(&p);

            // Create a random AABB
            let aabb = AABB::empty()
                .grow(&to_point(&a))
                .grow(&to_point(&b));

            // Create an AABB by using the index-access method
            let aabb_by_index = AABB::with_bounds(aabb[0], aabb[1]);

            // The AABBs should be the same
            aabb.contains(&point) == aabb_by_index.contains(&point)
        }
    }
}
