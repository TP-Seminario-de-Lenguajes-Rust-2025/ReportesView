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
