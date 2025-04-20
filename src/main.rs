/// ВАЖНО: все задания выполнять не обязательно. Что получится то получится сделать.

/// Задание 1
/// Почему фунция example1 зависает?
// Ответ:
// корутина a1 блокирует единственный поток пока не получит сообщение
// Варианты решения:
// - Mожно увеличить количество потоков в runtime до 2 хотя бы
// - Можно поменять порядок вызова rt.spawn(a1) и rt.spawn(a2)
// - Можно заменить try_recv на recv(асинхронный вариант) и тогда обе корутины будут смогут выполняться
// - Можно явно вызвать yield в корутине a1
fn example1() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        // Важно что тут 1 поток
        .worker_threads(1)
        .build()
        .unwrap();
    let (sd, mut rc) = tokio::sync::mpsc::unbounded_channel();

    let a1 = async move {
        // бесконечный цикл блокирует поток пока не получим сообщение
        loop {
            if let Ok(p) = rc.try_recv() {
                println!("{}", p);
                break;
            }
        }
    };
    // Запускаем корутину на tokio runtime и блокируем его
    let h1 = rt.spawn(a1);

    let a2 = async move {
        let _ = sd.send("message");
    };
    // Запускаем корутину на tokio runtime и она никогдан будет завершена потому что поток блокирован кортуиной на чтение
    let h2 = rt.spawn(a2);
    while !(h1.is_finished() || h2.is_finished()) {}

    println!("execution completed");
}

#[derive(Clone)]
struct Example2Struct {
    value: u64,
    ptr: *const u64,
}

/// Задание 2
/// Какое число тут будет распечатано 32 64 или 128 и почему?
// Ответ:
// Я бы сказал что это UB, ведь указатель t2.ptr указывает на место где хранилось t1.value.
// но из-за того что это все происходит в рамках стека одной функции и какие-то изменения по этому адресу произойдут только после выходны из функции,
// скоре всего вывод будет 64, потому что это последние значение t1.value
fn example2() {

    let num = 32; // num == 32, addr(num) == 0x0

    let mut t1 = Example2Struct {
        value: 64, // t1.value == 64, addr(t1.value) == 0x1
        ptr: &num, // t1.ptr == addr(num) == 0x0, *t1.ptr == 32
    };

    t1.ptr = &t1.value; // t1.ptr == addr(t1.value) == 0x1, *t1.ptr == 64

    // копирование value и ptr в новый объект на стеке
    // t2.value == 64, addr(t2.value) == 0x3
    // t2.ptr == addr(t1.value) == 0x1, *t2.ptr == 64
    let mut t2 = t1.clone();
    // Уничтожаем t1
    // Мы больше не можем получить доступ к t1.value и t1.ptr
    // но память на стеке осталась и никуда неделась
    // и t2.ptr до сих пор указывает на addr(t1.value) == 0x1
    drop(t1);


    t2.value = 128; // t2.value == 128, addr(t2.value) == 0x3

    unsafe {
        // t2.ptr == addr(t1.value) == 0x1, *t2.ptr == 64
        // Вывод будет 64, но это не очень безопасно посколько объект уничтожен
        // и если бы это был объект по сложнее чем u64 могло бы бомануть и пострадать соседская кошка
        println!("{}", t2.ptr.read());
    }

    println!("execution completed");
}

/// Задание 3
/// Почему время исполнения всех пяти заполнений векторов разное (под linux)?
// Ответ: Разобрал все варианты в readme.md
fn example3() {
    let capacity = 10000000u64;

    let start_time = std::time::Instant::now();
    let mut my_vec1 = Vec::new();
    for i in 0u64..capacity {
        my_vec1.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec2 = Vec::with_capacity(capacity as usize);
    for i in 0u64..capacity {
        my_vec2.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec3 = vec![6u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    for mut elem in my_vec3 {
        elem = 7u64;
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let my_vec4 = vec![0u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );
}

/// Задание 4
/// Почему такая разница во времени выполнения example4_async_mutex и example4_std_mutex?
// Ответ: разобрал варианты в readme.md
async fn example4_async_mutex(tokio_protected_value: std::sync::Arc<tokio::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *tokio_protected_value.clone().lock().await;
        value = 4;
    }
}

async fn example4_std_mutex(protected_value: std::sync::Arc<std::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *protected_value.clone().lock().unwrap();
        value = 4;
    }
}

fn example4() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();

    let mut tokio_protected_value = std::sync::Arc::new(tokio::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h2 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h3 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let protected_value = std::sync::Arc::new(std::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h2 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h3 = rt.spawn(example4_std_mutex(protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    println!("execution completed");
}

/// Задание 5
/// В чем ошибка дизайна? Каких тестов не хватает? Есть ли лишние тесты?
// Ответ: разобрал в readme.md и написал пример исправленного дизайна в mod example5_fixed
mod example5 {
    pub struct Triangle {
        pub a: (f32, f32),
        pub b: (f32, f32),
        pub c: (f32, f32),
        area: Option<f32>,
        perimeter: Option<f32>,
    }

    impl Triangle {
        //calculate area which is a positive number
        pub fn area(&mut self) -> f32 {
            if let Some(area) = self.area {
                area
            } else {
                self.area = Some(f32::abs(
                    (1f32 / 2f32) * (self.a.0 - self.c.0) * (self.b.1 - self.c.1)
                        - (self.b.0 - self.c.0) * (self.a.1 - self.c.1),
                ));
                self.area.unwrap()
            }
        }

        fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
            f32::sqrt((b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1))
        }

        //calculate perimeter which is a positive number
        pub fn perimeter(&mut self) -> f32 {
            if let Some(perimeter) = self.perimeter {
                return perimeter;
            } else {
                self.perimeter = Some(
                    Triangle::dist(self.a, self.b)
                        + Triangle::dist(self.b, self.c)
                        + Triangle::dist(self.c, self.a),
                );
                self.perimeter.unwrap()
            }
        }

        //new makes no guarantee for a specific values of a,b,c,area,perimeter at initialization
        pub fn new() -> Triangle {
            Triangle {
                a: (0f32, 0f32),
                b: (0f32, 0f32),
                c: (0f32, 0f32),
                area: None,
                perimeter: None,
            }
        }
    }
}

#[cfg(test)]
mod example5_tests {
    use super::example5::Triangle;

    #[test]
    fn test_area() {
        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 0f32);
        t.c = (0f32, 0f32);

        assert!(t.area() == 0f32);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1f32);
        t.c = (1f32, 0f32);

        assert!(t.area() == 0.5);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1000f32);
        t.c = (1000f32, 0f32);

        println!("{}",t.area());
    }

    #[test]
    fn test_perimeter() {
        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 0f32);
        t.c = (0f32, 0f32);

        assert!(t.perimeter() == 0f32);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1f32);
        t.c = (1f32, 0f32);

        assert!(t.perimeter() == 2f32 + f32::sqrt(2f32));
    }
}

mod example5_fixed {
    pub trait Point2d {
        fn distance(&self, other: &Self) -> f32;
    }

    pub trait Area {
        fn area(&self) -> f32;
    }

    pub trait Perimeter {
        fn perimeter(&self) -> f32;
    }

    pub type F32Point2d = (f32, f32);

    impl Point2d for F32Point2d {
        fn distance(&self, other: &Self) -> f32 {
            f32::sqrt(
                (self.0 - other.0) * (self.0 - other.0)
                    + (self.1 - other.1) * (self.1 - other.1),
            )
        }
    }

    pub struct Triangle<P: Point2d> {
        pub a: P,
        pub b: P,
        pub c: P,
    }

    impl <P: Point2d> Triangle<P> {
        pub fn new(a: P, b: P, c: P) -> Option<Triangle<P>> {
            let triangle = Self::new_unchecked(a, b, c);
            if triangle.area() == 0f32 {
                return None;
            }

            Some(triangle)
        }

        pub fn new_unchecked(a: P, b: P, c: P) -> Triangle<P> {
            Self { a, b, c }
        }
    }

    impl<P: Point2d> Area for Triangle<P> {
        fn area(&self) -> f32 {
            let a = self.a.distance(&self.b);
            let b = self.b.distance(&self.c);
            let c = self.b.distance(&self.a);

            let s = (a + b + c) / 2f32;

            (s * (s - a) * (s - b) * (s - c)).sqrt()
        }
    }

    impl<P: Point2d> Perimeter for Triangle<P> {
        fn perimeter(&self) -> f32 {
            self.a.distance(&self.b)
                + self.b.distance(&self.c)
                + self.c.distance(&self.a)
        }
    }
}

#[cfg(test)]
mod example5_fixed_tests {
    use rstest::rstest;
    use super::example5_fixed::*;

    fn check_equality(left: f32, right: f32) {
        assert!(
            (left - right).abs() < 0.000001f32,
            "{} != {}",
            left,
            right
        );
    }

    #[rstest]
    #[case(Triangle::new_unchecked((0f32, 0f32), (0f32, 0f32), (0f32, 0f32)), 0f32)]
    #[case(Triangle::new_unchecked((0f32, 0f32), (0f32, 1f32), (1f32, 0f32)), 0.5)]
    #[case(Triangle::new_unchecked((-1f32, -1f32), (-1f32, 1f32), (1f32, -1f32)), 2f32)]
    fn test_area(#[case] object: impl Area, #[case] expected: f32) {
        check_equality(object.area(), expected);
    }

    #[rstest]
    #[case(Triangle::new_unchecked((0f32, 0f32), (0f32, 0f32), (0f32, 0f32)), 0f32)]
    #[case(Triangle::new_unchecked((0f32, 0f32), (0f32, 1f32), (1f32, 0f32)), 2f32 + f32::sqrt(2f32))]
    #[case(Triangle::new_unchecked((-1f32, -1f32), (-1f32, 1f32), (1f32, -1f32)), 4f32 + 2f32 * f32::sqrt(2f32))]
    fn test_perimeter(#[case] object: impl Perimeter, #[case] expected: f32) {
        check_equality(object.perimeter(), expected);
    }

    #[rstest]
    #[case((0f32, 0f32), (0f32, 0f32), 0f32)]
    #[case((0f32, 0f32), (0f32, 1f32), 1f32)]
    #[case((0f32, 0f32), (1f32, 0f32), 1f32)]
    #[case((0f32, 0f32), (1f32, 1f32), f32::sqrt(2f32))]
    #[case((0f32, 0f32), (-1f32, -1f32), f32::sqrt(2f32))]
    #[case((1f32, 1f32), (2f32, 2f32), f32::sqrt(2f32))]
    #[case((-1f32, -1f32), (-2f32, -2f32), f32::sqrt(2f32))]
    fn test_distance(#[case] p1: F32Point2d, #[case] p2: F32Point2d, #[case] expected: f32) {
        check_equality(p1.distance(&p2), expected);
        check_equality(p2.distance(&p1), expected);
    }
}

fn main() {
    example3();
}