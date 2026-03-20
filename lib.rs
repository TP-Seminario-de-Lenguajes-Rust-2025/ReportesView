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

    //TODO: Los tipos de retorno son genericos. Hay que crear
    //      un struct que contenga producto_id, nombre del producto
    //      y cantidad total de ventas (entregadas).
    pub trait ConsultasProductos {
        fn _get_productos_mas_vendidos(&self, limit_to: u32) -> Vec<Producto>;
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
            usuarios_filtrados.sort_by(|mut a, mut b| {
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
        let c = Usuario::new(account_id(AccountKeyring::Charlie), String::from("Charlie"), String::from("Charlie"));
        usuarios.push(a);
        usuarios.push(b);
        usuarios.push(c);
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
        assert_eq!(ordenes_por_usuario.len(), usuarios.len());

        //assert!(matches!(ordenes_por_usuario.first(), &reporte_ordenes_usuario.first())); //no estoy seguro de esto
        assert_eq!(ordenes_por_usuario.first().unwrap().nombre_usuario, reporte_ordenes_usuario.first().unwrap().nombre_usuario, "nombre de usuario del primero de cada estructura deberia ser el mismo");
        assert_eq!(ordenes_por_usuario.first().unwrap().cantidad_ordenes, reporte_ordenes_usuario.first().unwrap().cantidad_ordenes, "cantidad de ordenes del primero de cada estructura deberia ser el mismo");
    }

    #[ink::test]
    fn test_get_cantidad_ordenes_por_usuario_vacio(){

    }

    #[ink::test]
    fn test_mejores_usuarios_por_rol(){

    }

    #[ink::test]
    fn test_usuarios_por_rol_sin_usuarios(){

    }

}