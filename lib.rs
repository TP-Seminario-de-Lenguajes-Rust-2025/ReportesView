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
        fn _get_cantidad_de_ordenes_por_usuario(&self) -> Vec<ReporteOrdenesUsuario>;

        fn _get_mejores_usuarios_por_rol(&self, target_role: &Rol) -> Vec<Usuario>; //separar por rol compra vender

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
            self._get_cantidad_de_ordenes_por_usuario()
        }

        #[ink(message)]
        pub fn get_mejores_usuarios_por_rol(&self, target_role: Rol) -> Vec<Usuario> {
            self._get_mejores_usuarios_por_rol(&target_role)
        }
    }

    impl ConsultasUsuarios for Reportes {
        fn _get_cantidad_de_ordenes_por_usuario(&self) -> Vec<ReporteOrdenesUsuario> {
            let usuarios = self.original.listar_usuarios();
            let ordenes = self.original.listar_ordenes();
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

        fn _get_mejores_usuarios_por_rol(&self, target_role: &Rol) -> Vec<Usuario> {
            let usuarios = self.original.listar_usuarios();
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
                Rol::Comprador => usuario.rating.calificacion_comprador,
                Rol::Vendedor => usuario.rating.calificacion_vendedor,
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
    use crate::contract::*;
    use crate::reportes::*;

    use ink::{
        env::{test::set_callee, DefaultEnvironment},
        primitives::AccountId,
    };
    use ink_e2e::{account_id, AccountKeyring};

    fn setup_sistema() -> Sistema {
        Sistema::new()
    }

    fn id_comprador() -> <DefaultEnvironment as ink::env::Environment>::AccountId {
        account_id(AccountKeyring::Alice)
    }

    fn id_vendedor() -> <DefaultEnvironment as ink::env::Environment>::AccountId {
        account_id(AccountKeyring::Bob)
    }

    fn set_caller(caller: AccountId) {
        ink::env::test::set_caller::<DefaultEnvironment>(caller);
    }

    fn build_testing_accounts() -> (AccountId, AccountId) {
        let id_comprador = id_comprador();
        let id_vendedor = id_vendedor();
        (id_comprador, id_vendedor)
    }

    fn build_testing_setup() -> (Sistema, AccountId, AccountId) {
        let mut app = setup_sistema();
        let (user_1, user_2) = build_testing_accounts();

        app._registrar_usuario(
            user_1,
            "user_name_1".to_string(),
            "user_email_1".to_string(),
            Rol::Comprador,
        )
        .expect("No se pudo registrar el usuario");
        app._registrar_usuario(
            user_2,
            "user_name_2".to_string(),
            "user_email_2".to_string(),
            Rol::Vendedor,
        )
        .expect("No se pudo registrar el usuario");

        (app, user_1, user_2)
    }

    //fn de test de agus olthoff

    fn registrar_comprador(
        sistema: &mut Sistema,
        id: <DefaultEnvironment as ink::env::Environment>::AccountId,
    ) {
        sistema
            ._registrar_usuario(
                id,
                "Comprador".into(),
                "comprador@gmail.com".into(),
                Rol::Comprador,
            )
            .unwrap();
    }
    fn registrar_vendedor(
        sistema: &mut Sistema,
        id: <DefaultEnvironment as ink::env::Environment>::AccountId,
    ) {
        sistema
            ._registrar_usuario(
                id,
                "Vendedor".into(),
                "vendedor@gmail.com".into(),
                Rol::Vendedor,
            )
            .unwrap();
    }

    fn agregar_categoria(sistema: &mut Sistema, nombre: &str) {
        sistema._registrar_categoria(nombre.into()).unwrap();
    }

    fn contrato_con_categorias_cargada() -> Sistema {
        let mut sist = Sistema::new();
        for i in 0..10 {
            let _ = sist._registrar_categoria(String::from(format!("categoria {}", i)));
        }
        return sist;
    }

    fn reportes_view(address: AccountId) -> Reportes {
        let reportes = Reportes::new(ink::env::account_id);
        reportes
    }


    #[ink::test]
    fn test_get_cantidad_ordenes_por_usuario_exitosa(){
        let (sistema, user1, user2) = build_testing_setup();
        assert_eq!(2,sistema.listar_usuarios().length());
    }
}