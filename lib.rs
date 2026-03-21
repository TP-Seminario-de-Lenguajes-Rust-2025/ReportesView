#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use super::*;
    use ink::env::call::FromAccountId;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use marketplacedescentralizado::prelude::*;

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Clone)]
    pub struct ReporteOrdenesUsuario {
        pub nombre_usuario: String,
        pub cantidad_ordenes: u32,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Clone)]
    pub struct ProductosVendidos {
        pub id_producto: u32,
        pub nombre_producto: String,
        pub cantidad_ventas: u32,
    }

    //TODO: Los tipos de retorno son genericos. Hay que crear
    //      un struct que contenga producto_id, nombre del producto
    //      y cantidad total de ventas (entregadas).
    pub trait ConsultasProductos {
        fn _get_productos_mas_vendidos(
            &self, 
            limit_to: u32,
            ordenes: Vec<Orden>,
            publicaciones: Vec<Publicacion>,
            productos: Vec<Producto>
        ) -> Vec<ProductosVendidos>;
    }

    //TODO: Los tipos de retorno son genericos. Hay que crear
    //      un struct que contenga categoria_id, nombre categoria
    //      y cantidad total de ventas (entregadas) de la categoria
    //      y calificacion promedio de la categoria. Se retorna un Vec.
    pub trait ConsultasCategorias {
        fn _get_estadisticas_por_categoria(&self, categoria: &str) -> Vec<String>;
    }

    //TODO: Los tipos de retorno son genericos. Hay que crear
    //      un struct que contenga usuario_id, nombre_usuario,
    //      y cantidad total ordenes (todas). Se retorna un Vec
    //      para (get_cantidad_de_ordenes_por_usuario).
    ///
    //      Despues, hay que crear un struct que contenga account_id,
    //      nombre del usuario y su reputacion. Se retorna el Vec
    //      ordenado DESC por reputacion de usuario (ver como ordenar, si
    ///     por str o por numerico)
    pub trait ConsultasUsuarios {
        fn _get_cantidad_de_ordenes_por_usuario(&self, usuarios: Vec<Usuario>, ordenes: Vec<Orden>) -> Vec<ReporteOrdenesUsuario>;

        fn _get_mejores_usuarios_por_rol(&self, target_role: &Rol, usuarios: Vec<Usuario>) -> Vec<Usuario>; //separar por rol compra vender

        fn _calcular_promedio(&self, usuario: &Usuario, rol: &Rol) -> u32;
    }

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

        /// Devuelve una lista de todos los usuarios registrados en el contrato original.
        #[ink(message)]
        pub fn get_cantidad_de_ordenes_por_usuario(&self) -> Vec<ReporteOrdenesUsuario> {
            let ordenes = self.get_ordenes();
            let usuarios = self.get_usuarios();
            self._get_cantidad_de_ordenes_por_usuario(usuarios, ordenes)
        }

        #[ink(message)]
        pub fn get_productos_mas_vendidos(&self, limit_to: u32) -> Vec<ProductosVendidos> {
            let ordenes = self.original.listar_ordenes();
            let publicaciones = self.original.listar_publicaciones();
            let productos = self.original.listar_productos();
            self._get_productos_mas_vendidos(limit_to, ordenes, publicaciones, productos)
        }

        #[ink(message)]
        pub fn get_mejores_usuarios_por_rol(&self, target_role: Rol) -> Vec<Usuario> {
            let usuarios = self.get_usuarios();
            self._get_mejores_usuarios_por_rol(&target_role, usuarios)
        }

        fn get_usuarios(&self) -> Vec<Usuario> {
            self.original.listar_usuarios(1, 500)
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
                                    nombre_real = prod.nombre.clone();
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
                if usuario.has_role(target_role.clone()) {
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
                Rol::Comprador => usuario.clone().rating.get_calificacion_comprador(),
                Rol::Vendedor => usuario.clone().rating.get_calificacion_vendedor(),
                _ => (0, 0),
            };
            if cantidad == 0 {
                0
            } else {
                puntos.checked_div(cantidad).unwrap_or(0)
            }
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
        reporte.push(ReporteOrdenesUsuario { nombre_usuario: (String::from("Alice")), cantidad_ordenes: 2 });
        reporte.push(ReporteOrdenesUsuario { nombre_usuario: (String::from("Bob")), cantidad_ordenes: 1 });
        reporte.push(ReporteOrdenesUsuario { nombre_usuario: (String::from("Charlie")), cantidad_ordenes: 1 });
        reporte.push(ReporteOrdenesUsuario { nombre_usuario: (String::from("Dave")), cantidad_ordenes: 0 });
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

}