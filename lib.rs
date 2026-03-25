#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use super::*;
    use ink::env::call::FromAccountId;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use marketplacedescentralizado::prelude::*;
    use core::{
        ops::{Div, Rem},
    };
    use scale_info::prelude::format;

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Clone)]
    pub struct ReporteOrdenesUsuario {
        pub id_usuario: AccountId,
        pub nombre_usuario: String,
        pub cantidad_ordenes: u32,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct EstadisticasCategoria {
        pub categoria_id: u32,
        pub nombre_categoria: String,
        pub ventas_entregadas: u32,
        pub calificacion_promedio: String, 
    }
  
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Clone)]
    pub struct ProductosVendidos {
        pub id_producto: u32,
        pub nombre_producto: String,
        pub cantidad_ventas: u32,
    }


    pub trait ConsultasProductos {
        fn _get_productos_mas_vendidos(
            &self, 
            limit_to: u32,
            ordenes: Vec<Orden>,
            publicaciones: Vec<Publicacion>,
            productos: Vec<Producto>
        ) -> Vec<ProductosVendidos>;
    }

    pub trait ConsultasCategorias {
        fn _get_estadisticas_por_categoria(
            &self, 
            categorias: Vec<Categoria>,
            productos: Vec<Producto>,
            publicaciones: Vec<Publicacion>,
            ordenes: Vec<Orden>,
        ) -> Vec<EstadisticasCategoria>;
    }

    pub trait ConsultasUsuarios {
        fn _get_cantidad_de_ordenes_por_usuario(&self, usuarios: Vec<Usuario>, ordenes: Vec<Orden>) -> Vec<ReporteOrdenesUsuario>;

        fn _get_mejores_usuarios_por_rol(&self, target_role: &Rol, usuarios: Vec<Usuario>) -> Vec<Usuario>; 

        fn _calcular_promedio(&self, usuario: &Usuario, rol: &Rol) -> u32;
    }

    /// Estructura con la referencia al contrato original
    #[ink(storage)]
    pub struct Reportes {
        original: SistemaRef,
    }

    impl Reportes {
        #[ink(constructor)]
        pub fn new(address: AccountId) -> Self {
            let original = SistemaRef::from_account_id(address);
            Self { original }
        }

        /// Devuelve una lista de las ordenes tiene registradas por cada usuario
        #[ink(message)]
        pub fn get_cantidad_de_ordenes_por_usuario(&self) -> Vec<ReporteOrdenesUsuario> {
            let ordenes = self.get_ordenes();
            let usuarios = self.get_usuarios();
            self._get_cantidad_de_ordenes_por_usuario(usuarios, ordenes)
        }

        /// Devuelve una lista ordenada por los productos más vendidos
        /// 
        /// # Parámetro
        /// - `limit_to`: la cantidad de productos a devolver
        #[ink(message)]
        pub fn get_productos_mas_vendidos(&self, limit_to: u32) -> Vec<ProductosVendidos> {
            let ordenes = self.original.listar_ordenes();
            let publicaciones = self.original.listar_publicaciones();
            let productos = self.original.listar_productos();
            self._get_productos_mas_vendidos(limit_to, ordenes, publicaciones, productos)
        }

        /// Devuelve los 5 usuarios mejor calificados segun el rol provisto
        /// 
        /// # Parámetro
        /// - `target_role`: el rol por el cual filtrar. En caso de "Ambos", no se filtra
        #[ink(message)]
        pub fn get_mejores_usuarios_por_rol(&self, target_role: Rol) -> Vec<Usuario> {
            let usuarios = self.get_usuarios();
            self._get_mejores_usuarios_por_rol(&target_role, usuarios)
        }

        /// Devuelve el total de ventas y la calificación promedio de los productos de cada categoría
        #[ink(message)]
        pub fn get_estadisticas_por_categoria(&self) -> Vec<EstadisticasCategoria> {
            let categorias = self.original.listar_categorias();
            let productos = self.original.listar_productos();
            let publicaciones = self.original.listar_publicaciones();
            let ordenes = self.get_ordenes();

            self._get_estadisticas_por_categoria(categorias, productos, publicaciones, ordenes)
        }

        fn get_usuarios(&self) -> Vec<Usuario> {
            self.original.listar_usuarios(0, 0)
        }

        fn get_ordenes(&self) -> Vec<Orden> {
            self.original.listar_ordenes()
        }
    }

    impl ConsultasProductos for Reportes {
        fn _get_productos_mas_vendidos(&self, limit_to: u32, ordenes: Vec<Orden>, publicaciones: Vec<Publicacion>, productos: Vec<Producto>) -> Vec<ProductosVendidos> {
            let mut lista_vendidos: Vec<ProductosVendidos> = Vec::new();


            for orden in ordenes {
                if orden.get_status() == EstadoOrden::Recibida { 
                    
                    let id_pub = orden.get_id_publicacion();
                    let cantidad = orden.get_cantidad();

                    let mut id_producto_real = None;
                    for publi in &publicaciones {
                        if publi.get_id() == id_pub {
                            id_producto_real = Some(publi.get_id_producto());
                            break;
                        }
                    }

                    if let Some(id_prod) = id_producto_real {
                        let mut encontrado = false;

                        for item in lista_vendidos.iter_mut() {
                            if item.id_producto == id_prod {
                                item.cantidad_ventas = item.cantidad_ventas.saturating_add(cantidad);
                                encontrado = true;
                                break;
                            }
                        }

                        if !encontrado {
                            let mut nombre_real = String::from("Producto Desconocido");
                            for prod in &productos {
                                if prod.get_id() == id_prod {
                                    nombre_real = prod.get_nombre();
                                    break;
                                }
                            }
                            
                            lista_vendidos.push(ProductosVendidos {
                                id_producto: id_prod,
                                nombre_producto: nombre_real,
                                cantidad_ventas: cantidad,
                            });
                        }
                    }
                }
            }

            lista_vendidos.sort_by(|a, b| b.cantidad_ventas.cmp(&a.cantidad_ventas));

            let mut resultado_final = Vec::new();
            for (index, item) in lista_vendidos.into_iter().enumerate() {
                if index as u32 >= limit_to {
                    break;
                }
                resultado_final.push(item);
            }

            resultado_final
        }
    }

    impl ConsultasUsuarios for Reportes {
        fn _get_cantidad_de_ordenes_por_usuario(&self, usuarios: Vec<Usuario>, ordenes: Vec<Orden>) -> Vec<ReporteOrdenesUsuario> {
            let mut reporte = Vec::new();

            for usuario in usuarios {
                let mut contador: u32 = 0;
                for orden in &ordenes {
                    if orden.get_id_comprador() == usuario.get_id() {
                        contador = contador.saturating_add(1);
                    }
                }
                let item = ReporteOrdenesUsuario {
                    id_usuario : usuario.get_id(),
                    nombre_usuario: usuario.get_name(),
                    cantidad_ordenes: contador,
                };
                reporte.push(item);
            }
            reporte
        }

        fn _get_mejores_usuarios_por_rol(&self, target_role: &Rol, usuarios: Vec<Usuario>) -> Vec<Usuario> {
            let mut usuarios_filtrados = Vec::new();

            //aca filtro usuarios que tengan el target role
            for usuario in usuarios {
                if matches!(target_role, Rol::Ambos) || usuario.has_role(target_role.clone()) {
                    usuarios_filtrados.push(usuario);
                }
            }
            //ordeno  por promedio de mayor a menos
            usuarios_filtrados.sort_by(|a, b| {
                let prom_a = self._calcular_promedio(a, &target_role);
                let prom_b = self._calcular_promedio(b, &target_role);
                prom_b.cmp(&prom_a)
            });
            //me quedo con solo los 5 primeros
            let mut top_5 = Vec::new();
            for (count, u) in usuarios_filtrados.into_iter().enumerate() {
                if count >= 5 {
                    break;
                }
                top_5.push(u);
            }
            top_5
        }

        //funcion auxialiar para calcular promedio
        fn _calcular_promedio(&self, usuario: &Usuario, rol: &Rol) -> u32 {
            let (puntos, cantidad) = match rol {
                Rol::Comprador => usuario.clone().get_calificacion_comprador(),
                Rol::Vendedor => usuario.clone().get_calificacion_vendedor(),
                _ => (0, 0),
            };
            if cantidad == 0 {
                0
            } else {
                puntos.checked_div(cantidad).unwrap_or(0)
            }
        }
    }

    impl ConsultasCategorias for Reportes {
        fn _get_estadisticas_por_categoria(
            &self, 
            categorias: Vec<Categoria>,
            productos: Vec<Producto>,
            publicaciones: Vec<Publicacion>,
            ordenes: Vec<Orden>,
        ) -> Vec<EstadisticasCategoria> {
            let mut reporte = Vec::new();
            
            for cat in categorias {
                let mut ventas_entregadas: u32 = 0;
                let mut suma_calificaciones: u32 = 0;
                let mut cantidad_calificaciones: u32 = 0;
                
                for orden in &ordenes {

                    // se cuentan solamente las ordenes recibidas
                    if orden.get_status() == EstadoOrden::Recibida {
                        let mut pertenece_a_cat = false;

                        // se busca la categoria a partir de orden (Orden -> id Publicacion -> id Producto -> id Categoria)
                        for publi in &publicaciones {
                            if publi.get_id() == orden.get_id_publicacion() {
                                for prod in &productos {
                                    if prod.get_id() == publi.get_id_producto() && prod.get_id_categoria() == cat.get_id() {
                                        pertenece_a_cat = true;
                                        break;
                                    }
                                }
                                break;
                            }
                        }

                        if pertenece_a_cat {
                            ventas_entregadas = ventas_entregadas.saturating_add(orden.get_cantidad());
                            // get_calificacion_vendedor devuelve la calificacion que recibe el vendedor (i.e. la que recibe el producto)
                            if let Some(cal) = orden.get_calificacion_vendedor() {
                                suma_calificaciones = suma_calificaciones.saturating_add(cal as u32); // casteo de u8 a u32 
                                cantidad_calificaciones = cantidad_calificaciones.saturating_add(1);
                            }
                        }
                    }
                }
                
                let calificacion_promedio = if cantidad_calificaciones > 0 {
                    (suma_calificaciones.saturating_mul(10)).checked_div(cantidad_calificaciones).unwrap_or(0)
                } else { 0 };

                reporte.push(EstadisticasCategoria {
                    categoria_id: cat.get_id(),
                    nombre_categoria: cat.get_nombre(),
                    ventas_entregadas,
                    calificacion_promedio: String::from(format!("{entero},{decimal}", entero = calificacion_promedio.div(10), decimal = calificacion_promedio.rem(10)))
                });
            }
            reporte
        }
    }
}

#[cfg(test)]
mod tests {
    //use marketplacedescentralizado::{prelude::*, contract::*};
    use marketplacedescentralizado::prelude::*;
    use crate::reportes::*;

    use ink::{
        env::{test::set_callee, DefaultEnvironment},
        primitives::AccountId,
    };
    use ink_e2e::{account_id, AccountKeyring};

    fn generar_vec_usuarios() -> Vec<Usuario> {
        let mut usuarios = Vec::new();

        let a = Usuario::new(account_id(AccountKeyring::Alice), String::from("Alice"), String::from("alice@email.com"));
        let b = Usuario::new(account_id(AccountKeyring::Bob), String::from("Bob"), String::from("bob@email.com"));
        let c = Usuario::new(account_id(AccountKeyring::Charlie), String::from("Charlie"), String::from("charlie@mail.com"));
        let d = Usuario::new(account_id(AccountKeyring::Dave), String::from("Dave"), String::from("dave@aol.com"));
        usuarios.push(a);
        usuarios.push(b);
        usuarios.push(c);
        usuarios.push(d);
        usuarios
    }

    fn generar_vec_orden() -> Vec<Orden> {
        let mut ordenes = Vec::new();

        //alice vende 1 a bob 
        let o1 = Orden::new(1, 1, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Bob), 1, 1);

        //alice compra 2 a charlie
        let o2 = Orden::new(2, 2, account_id(AccountKeyring::Charlie), account_id(AccountKeyring::Alice), 1, 1);
        let o3 = Orden::new(3, 3, account_id(AccountKeyring::Charlie), account_id(AccountKeyring::Alice), 1, 1);

        //charlie compra a bob
        let o4 = Orden::new(4, 4, account_id(AccountKeyring::Bob), account_id(AccountKeyring::Charlie), 1, 1);

        ordenes.push(o1);
        ordenes.push(o2);
        ordenes.push(o3);
        ordenes.push(o4);
        ordenes
    }

    fn generar_reporte_ordenes() -> Vec<ReporteOrdenesUsuario>{
        let mut reporte = Vec::new();
        reporte.push(ReporteOrdenesUsuario { id_usuario: account_id(AccountKeyring::Alice), nombre_usuario: (String::from("Alice")), cantidad_ordenes: 2 });
        reporte.push(ReporteOrdenesUsuario { id_usuario: account_id(AccountKeyring::Bob), nombre_usuario: (String::from("Bob")), cantidad_ordenes: 1 });
        reporte.push(ReporteOrdenesUsuario { id_usuario: account_id(AccountKeyring::Charlie), nombre_usuario: (String::from("Charlie")), cantidad_ordenes: 1 });
        reporte.push(ReporteOrdenesUsuario { id_usuario: account_id(AccountKeyring::Dave), nombre_usuario: (String::from("Dave")), cantidad_ordenes: 0 });
        reporte
    }

    fn setup_entorno() -> (Reportes, Vec<Usuario>, Vec<Orden>, Vec<ReporteOrdenesUsuario>){
        let reportes = Reportes::new(ink::env::account_id::<DefaultEnvironment>());
        let usuarios = generar_vec_usuarios();
        let ordenes = generar_vec_orden();
        let reporte_ordenes_usuario = generar_reporte_ordenes();
        (reportes, usuarios, ordenes, reporte_ordenes_usuario)
    }

    #[ink::test]
    fn test_get_cantidad_ordenes_por_usuario_exitosa(){
        let (reportes, usuarios, ordenes, reporte_ordenes_usuario) = setup_entorno();

        let ordenes_por_usuario = reportes._get_cantidad_de_ordenes_por_usuario(usuarios.clone(), ordenes.clone());
        assert_eq!(ordenes_por_usuario.len(), usuarios.len(), "largo de los vectores deberia ser igual");


        assert_eq!(ordenes_por_usuario.first().unwrap().nombre_usuario, reporte_ordenes_usuario.first().unwrap().nombre_usuario, "nombre de usuario del primero de cada estructura deberia ser el mismo");
        assert_eq!(ordenes_por_usuario.first().unwrap().cantidad_ordenes, reporte_ordenes_usuario.first().unwrap().cantidad_ordenes, "cantidad de ordenes del primero de cada estructura deberia ser la mismo");

        assert_eq!(ordenes_por_usuario.clone().pop().unwrap().nombre_usuario, reporte_ordenes_usuario.clone().pop().unwrap().nombre_usuario, "nombre de usuario del ultimo usuario de ambas estructuras deberia ser igual");
        assert_eq!(ordenes_por_usuario.clone().pop().unwrap().cantidad_ordenes, reporte_ordenes_usuario.clone().pop().unwrap().cantidad_ordenes, "cantidada de ordenes del ultimo usuario de ambas estructuras deberia ser 0");
    }

    #[ink::test]
    fn test_get_cantidad_ordenes_por_usuario_vacio(){
        let (reportes, _usuarios, _ordenes, _reporte_ordenes_usuario) = setup_entorno();

        let usuarios_vacios = Vec::new();
        let ordenes_vacias = Vec::new();

        let resultado1 = reportes._get_cantidad_de_ordenes_por_usuario(usuarios_vacios, _ordenes.clone());
        assert_eq!(resultado1.len(), 0, "Si no hay usuarios, el reporte debe estar vacío");

        let resultado2 = reportes._get_cantidad_de_ordenes_por_usuario(_usuarios.clone(), ordenes_vacias);
        assert_eq!(resultado2.len(), _usuarios.len(), "Debe haber una entrada por usuario aunque no tengan ordenes");
        for reporte in resultado2 {
            assert_eq!(reporte.cantidad_ordenes, 0, "Las ordenes deben ser 0 para todos en coleccion vacia de ordenes");
        }
    }

    #[ink::scale_derive(Encode)]
    struct MockRating {
        calificacion_comprador: (u32, u32),
        calificacion_vendedor: (u32, u32),
    }

    #[ink::scale_derive(Encode)]
    struct MockUsuario {
        id: AccountId,
        nombre: String,
        mail: String,
        rating: MockRating,
        roles: Vec<Rol>,
    }

    fn create_mock_usuario(id: AccountId, nombre: &str, mail: &str, roles: Vec<Rol>, c_comp: (u32, u32), c_vend: (u32, u32)) -> Usuario {
        let mock_rating = MockRating {
            calificacion_comprador: c_comp,
            calificacion_vendedor: c_vend,
        };
        let mock = MockUsuario {
            id,
            nombre: String::from(nombre),
            mail: String::from(mail),
            rating: mock_rating,
            roles,
        };
        let encoded = ink::scale::Encode::encode(&mock);
        ink::scale::Decode::decode(&mut &encoded[..]).unwrap()
    }

    #[ink::test]
    fn test_mejores_usuarios_por_rol(){
        let (reportes, _, _, _) = setup_entorno();

        // Creamos usuarios con puntajes especificos (Valor Acumulado, Cantidad de Reseñas)
        // Promedio Vendedor = (Acumulado) / Cantidad
        let u1 = create_mock_usuario(account_id(AccountKeyring::Alice), "Alice", "a@a.com", vec![Rol::Vendedor], (0, 0), (50, 10)); // Prom: 5
        let u2 = create_mock_usuario(account_id(AccountKeyring::Bob), "Bob", "b@b.com", vec![Rol::Vendedor], (0, 0), (20, 10)); // Prom: 2
        let u3 = create_mock_usuario(account_id(AccountKeyring::Charlie), "Charlie", "c@c.com", vec![Rol::Vendedor], (0, 0), (40, 10)); // Prom: 4
        let u4 = create_mock_usuario(account_id(AccountKeyring::Dave), "Dave", "d@d.com", vec![Rol::Vendedor], (0, 0), (10, 10)); // Prom: 1
        let u5 = create_mock_usuario(account_id(AccountKeyring::Eve), "Eve", "e@e.com", vec![Rol::Vendedor, Rol::Comprador], (0, 0), (30, 10)); // Prom: 3
        let u6 = create_mock_usuario(account_id(AccountKeyring::Ferdie), "Ferdie", "f@f.com", vec![Rol::Vendedor], (0, 0), (60, 10)); // Prom: 6
        
        let usuarios = vec![u1, u2, u3, u4, u5, u6];

        let mejores = reportes._get_mejores_usuarios_por_rol(&Rol::Vendedor, usuarios);
        
        // Verifica que no exceda 5
        assert_eq!(mejores.len(), 5);

        // Verifica el ordenamiento descendente correcto: u6 (6), u1 (5), u3 (4), u5 (3), u2 (2). u4 (1) queda fuera.
        assert_eq!(mejores[0].get_name(), "Ferdie");
        assert_eq!(mejores[1].get_name(), "Alice");
        assert_eq!(mejores[4].get_name(), "Bob");
    }

    #[ink::test]
    fn test_usuarios_por_rol_sin_usuarios(){
        let (reportes, _, _, _) = setup_entorno();
        let usuarios = Vec::new();

        let mejores = reportes._get_mejores_usuarios_por_rol(&Rol::Comprador, usuarios);
        assert_eq!(mejores.len(), 0, "No debe retornar nada al pasar un vector vacio");
    }

    // ==========================================
    // MOCK PARA SALTEAR LA PRIVACIDAD DE ORDEN
    // ==========================================

    #[ink::scale_derive(Encode)]
    struct MockOrden {
        id: u32,
        id_publicacion: u32,
        id_vendedor: AccountId,
        id_comprador: AccountId,
        status: EstadoOrden,
        cantidad: u32,
        precio_total: u128,
        cal_vendedor: Option<u8>,
        cal_comprador: Option<u8>,
    }

    // Helper que usa tu mismo truco de serialización para crear una Orden con cualquier estado
    fn crear_orden_mock(
        id: u32,
        id_publicacion: u32,
        id_vendedor: AccountId,
        id_comprador: AccountId,
        cantidad: u32,
        status: EstadoOrden,
    ) -> Orden {
        let mock = MockOrden {
            id,
            id_publicacion,
            id_vendedor,
            id_comprador,
            status,
            cantidad,
            precio_total: 1000, // Valor irrelevante para este test
            cal_vendedor: None,
            cal_comprador: None,
        };
        let encoded = ink::scale::Encode::encode(&mock);
        ink::scale::Decode::decode(&mut &encoded[..]).unwrap()
    }

    // ==========================================
    // HELPERS PARA GENERAR VECTORES DE PRUEBA
    // ==========================================

    fn generar_productos_mock() -> Vec<Producto> {
        vec![
            Producto::new(0, account_id(AccountKeyring::Alice), "Zapatillas".into(), "Desc".into(), 1, 10),
            Producto::new(1, account_id(AccountKeyring::Alice), "Remera".into(), "Desc".into(), 1, 10),
            Producto::new(2, account_id(AccountKeyring::Bob), "Pantalon".into(), "Desc".into(), 1, 10),
        ]
    }

    fn generar_publicaciones_mock() -> Vec<Publicacion> {
        vec![
            Publicacion::new(0, 0, account_id(AccountKeyring::Alice), 10, 100), // pub 0 -> prod 0 (Zapatillas)
            Publicacion::new(1, 1, account_id(AccountKeyring::Alice), 10, 50),  // pub 1 -> prod 1 (Remera)
            Publicacion::new(2, 2, account_id(AccountKeyring::Bob), 10, 200),   // pub 2 -> prod 2 (Pantalon)
            Publicacion::new(3, 0, account_id(AccountKeyring::Alice), 5, 120),  // pub 3 -> prod 0 (Zapatillas)
        ]
    }

    fn generar_ordenes_mock() -> Vec<Orden> {
        // Zapatillas (Pub 0 y 3): 2 ventas + 3 ventas = 5 en total. Ambas RECIBIDAS.
        let o1 = crear_orden_mock(0, 0, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 2, EstadoOrden::Recibida);
        let o2 = crear_orden_mock(1, 3, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Dave), 3, EstadoOrden::Recibida);

        // Remera (Pub 1): 1 venta RECIBIDA y 1 venta PENDIENTE (la pendiente de 10 unidades se debe ignorar)
        let o3 = crear_orden_mock(2, 1, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 1, EstadoOrden::Recibida);
        let o4 = crear_orden_mock(3, 1, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Dave), 10, EstadoOrden::Pendiente);

        // Pantalon (Pub 2): 4 ventas RECIBIDAS
        let o5 = crear_orden_mock(4, 2, account_id(AccountKeyring::Bob), account_id(AccountKeyring::Charlie), 4, EstadoOrden::Recibida);

        vec![o1, o2, o3, o4, o5]
    }

    // ==========================================
    // TESTS UNITARIOS: PRODUCTOS MAS VENDIDOS
    // ==========================================

    #[ink::test]
    fn test_get_productos_mas_vendidos_exito_y_ordenamiento() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let productos = generar_productos_mock();
        let publicaciones = generar_publicaciones_mock();
        let ordenes = generar_ordenes_mock();

        // Pedimos hasta 10 productos
        let top = reportes._get_productos_mas_vendidos(10, ordenes, publicaciones, productos);

        // Debería haber 3 productos únicos con ventas concretadas
        assert_eq!(top.len(), 3, "Deberían encontrarse 3 productos con ventas finalizadas");

        // El orden debe ser descendente estricto: Zapatillas(5), Pantalon(4), Remera(1)
        assert_eq!(top[0].nombre_producto, "Zapatillas");
        assert_eq!(top[0].cantidad_ventas, 5, "Zapatillas debe sumar 5 ventas combinadas");

        assert_eq!(top[1].nombre_producto, "Pantalon");
        assert_eq!(top[1].cantidad_ventas, 4);

        assert_eq!(top[2].nombre_producto, "Remera");
        assert_eq!(top[2].cantidad_ventas, 1, "La remera solo debe tener 1 venta, ignorando la orden Pendiente");
    }

    #[ink::test]
    fn test_get_productos_mas_vendidos_aplica_limite() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let productos = generar_productos_mock();
        let publicaciones = generar_publicaciones_mock();
        let ordenes = generar_ordenes_mock();

        // Le pedimos solo el TOP 2
        let top = reportes._get_productos_mas_vendidos(2, ordenes, publicaciones, productos);

        assert_eq!(top.len(), 2, "El resultado debe truncarse a 2 elementos");
        assert_eq!(top[0].nombre_producto, "Zapatillas");
        assert_eq!(top[1].nombre_producto, "Pantalon");
    }

    #[ink::test]
    fn test_get_productos_mas_vendidos_solo_cuenta_recibidas() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let productos = generar_productos_mock();
        let publicaciones = generar_publicaciones_mock();
        
        // Creamos ordenes que son SOLO pendientes o canceladas
        let ordenes = vec![
            crear_orden_mock(0, 0, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 5, EstadoOrden::Pendiente),
            crear_orden_mock(1, 1, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Dave), 2, EstadoOrden::Cancelada),
        ];

        let top = reportes._get_productos_mas_vendidos(10, ordenes, publicaciones, productos);

        assert_eq!(top.len(), 0, "Si no hay ventas 'Recibidas', el vector final debe estar vacío");
    }

    #[ink::test]
    fn test_get_productos_mas_vendidos_listas_vacias() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        
        // Simular bases de datos vacías para asegurar que no tire panic
        let top = reportes._get_productos_mas_vendidos(10, vec![], vec![], vec![]);

        assert_eq!(top.len(), 0, "Debe manejar correctamente las colecciones vacías sin crashear");
    }

    // ==========================================
    // TESTS UNITARIOS: ESTADISTICAS POR CATEGORIA
    // ==========================================

    // version de crear_orden_mock que permite setear calificaciones
    fn crear_orden_mock_con_cal(
        id: u32,
        id_publicacion: u32,
        id_vendedor: AccountId,
        id_comprador: AccountId,
        cantidad: u32,
        status: EstadoOrden,
        cal_vendedor: Option<u8>,
        cal_comprador: Option<u8>,
    ) -> Orden {
        let mock = MockOrden {
            id,
            id_publicacion,
            id_vendedor,
            id_comprador,
            status,
            cantidad,
            precio_total: 1000,
            cal_vendedor,
            cal_comprador,
        };
        let encoded = ink::scale::Encode::encode(&mock);
        ink::scale::Decode::decode(&mut &encoded[..]).unwrap()
    }

    // categorias para usar en los tests de estadisticas
    fn generar_categorias_test() -> Vec<Categoria> {
        vec![
            Categoria::new(0, "Indumentaria".into()),
            Categoria::new(1, "Electronica".into()),
        ]
    }

    // productos repartidos en las 2 categorias
    fn generar_productos_por_categoria() -> Vec<Producto> {
        vec![
            // categoria 0 (Indumentaria)
            Producto::new(0, account_id(AccountKeyring::Alice), "Remera".into(), "Desc".into(), 0, 50),
            Producto::new(1, account_id(AccountKeyring::Alice), "Pantalon".into(), "Desc".into(), 0, 30),
            // categoria 1 (Electronica)
            Producto::new(2, account_id(AccountKeyring::Bob), "Auriculares".into(), "Desc".into(), 1, 20),
        ]
    }

    fn generar_publicaciones_por_categoria() -> Vec<Publicacion> {
        vec![
            Publicacion::new(0, 0, account_id(AccountKeyring::Alice), 50, 100),  // pub 0 -> Remera
            Publicacion::new(1, 1, account_id(AccountKeyring::Alice), 30, 200),  // pub 1 -> Pantalon
            Publicacion::new(2, 2, account_id(AccountKeyring::Bob), 20, 500),    // pub 2 -> Auriculares
        ]
    }

    #[ink::test]
    fn test_estadisticas_por_categoria_caso_normal() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let categorias = generar_categorias_test();
        let productos = generar_productos_por_categoria();
        let publicaciones = generar_publicaciones_por_categoria();

        // armo ordenes recibidas con calificacion
        // 2 ordenes de Indumentaria (pub 0 y 1), 1 de Electronica (pub 2)
        let ordenes = vec![
            crear_orden_mock_con_cal(0, 0, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 3, EstadoOrden::Recibida, Some(4), None),
            crear_orden_mock_con_cal(1, 1, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Dave), 2, EstadoOrden::Recibida, Some(2), None),
            crear_orden_mock_con_cal(2, 2, account_id(AccountKeyring::Bob), account_id(AccountKeyring::Charlie), 1, EstadoOrden::Recibida, Some(5), None),
        ];

        let stats = reportes._get_estadisticas_por_categoria(categorias, productos, publicaciones, ordenes);

        // tiene que haber una entrada por categoria
        assert_eq!(stats.len(), 2);

        // Indumentaria: 3 + 2 = 5 ventas, promedio cal = (4+2)*10/2 = 30
        assert_eq!(stats[0].nombre_categoria, "Indumentaria");
        assert_eq!(stats[0].ventas_entregadas, 5);
        assert_eq!(stats[0].calificacion_promedio, String::from("3,0")); // (4+2)*10 / 2

        // Electronica: 1 venta, promedio cal = 5*10/1 = 50
        assert_eq!(stats[1].nombre_categoria, "Electronica");
        assert_eq!(stats[1].ventas_entregadas, 1);
        assert_eq!(stats[1].calificacion_promedio, String::from("5,0"));
    }

    #[ink::test]
    fn test_estadisticas_por_categoria_sin_ordenes() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let categorias = generar_categorias_test();
        let productos = generar_productos_por_categoria();
        let publicaciones = generar_publicaciones_por_categoria();

        // sin ordenes -> todo en 0
        let stats = reportes._get_estadisticas_por_categoria(categorias, productos, publicaciones, vec![]);

        assert_eq!(stats.len(), 2);
        for s in &stats {
            assert_eq!(s.ventas_entregadas, 0);
            assert_eq!(s.calificacion_promedio, String::from("0,0"));
        }
    }

    #[ink::test]
    fn test_estadisticas_por_categoria_ignora_no_recibidas() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let categorias = generar_categorias_test();
        let productos = generar_productos_por_categoria();
        let publicaciones = generar_publicaciones_por_categoria();

        // todas las ordenes estan pendientes o canceladas, no deberian contar
        let ordenes = vec![
            crear_orden_mock_con_cal(0, 0, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 5, EstadoOrden::Pendiente, Some(3), None),
            crear_orden_mock_con_cal(1, 2, account_id(AccountKeyring::Bob), account_id(AccountKeyring::Dave), 2, EstadoOrden::Cancelada, Some(5), None),
        ];

        let stats = reportes._get_estadisticas_por_categoria(categorias, productos, publicaciones, ordenes);

        for s in &stats {
            assert_eq!(s.ventas_entregadas, 0, "no deberia contar ventas de ordenes que no estan recibidas");
            assert_eq!(s.calificacion_promedio, String::from("0,0"));
        }
    }

    #[ink::test]
    fn test_estadisticas_por_categoria_sin_calificaciones() {
        let reportes = Reportes::new(account_id(AccountKeyring::Alice));
        let categorias = generar_categorias_test();
        let productos = generar_productos_por_categoria();
        let publicaciones = generar_publicaciones_por_categoria();

        // ordenes recibidas pero nadie califico
        let ordenes = vec![
            crear_orden_mock_con_cal(0, 0, account_id(AccountKeyring::Alice), account_id(AccountKeyring::Charlie), 4, EstadoOrden::Recibida, None, None),
            crear_orden_mock_con_cal(1, 2, account_id(AccountKeyring::Bob), account_id(AccountKeyring::Dave), 2, EstadoOrden::Recibida, None, None),
        ];

        let stats = reportes._get_estadisticas_por_categoria(categorias, productos, publicaciones, ordenes);

        // las ventas se suman igual, pero el promedio queda en 0 porque no hay calificaciones
        assert_eq!(stats[0].ventas_entregadas, 4); // Indumentaria
        assert_eq!(stats[0].calificacion_promedio, String::from("0,0"));

        assert_eq!(stats[1].ventas_entregadas, 2); // Electronica
        assert_eq!(stats[1].calificacion_promedio, String::from("0,0"));
    }

}
