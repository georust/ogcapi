use anyhow::anyhow;

#[cfg(feature = "movingfeatures")]
use ogcapi_types::movingfeatures::{
    temporal_complex_geometry::TemporalComplexGeometry,
    temporal_geometry::TemporalGeometry, temporal_primitive_geometry::TemporalPrimitiveGeometry,
};

#[cfg(feature = "movingfeatures")]
use crate::{FeatureTransactions, TemporalGeometryTransactions};

#[cfg(feature = "movingfeatures")]
#[async_trait::async_trait]
impl<T> TemporalGeometryTransactions for T
where
    T: FeatureTransactions,
{
    async fn create_temporal_geometry(
        &self,
        collection: &str,
        m_feature_id: &str,
        temporal_geometry: &TemporalPrimitiveGeometry,
    ) -> anyhow::Result<String> {
        let crs = temporal_geometry
            .crs
            .clone()
            .unwrap_or_default()
            .try_into()
            .map_err(|e: String| anyhow!(e))?;
        let mut feature = self
            .read_feature(collection, m_feature_id, &crs)
            .await?
            .ok_or(anyhow!("Feature not found!"))?;
        let mut temporal_geometry = temporal_geometry.clone();
        let id = match feature.temporal_geometry {
            None => {
                temporal_geometry.id = Some(0.to_string());
                feature.temporal_geometry =
                    Some(TemporalGeometry::Primitive(temporal_geometry.clone()));
                temporal_geometry.id.unwrap()
            }
            Some(TemporalGeometry::Primitive(tg)) => {
                temporal_geometry.id = Some(1.to_string());
                feature.temporal_geometry =
                    Some(TemporalGeometry::Complex(TemporalComplexGeometry {
                        prisms: vec![tg, temporal_geometry.clone()],
                        r#type: Default::default(),
                        crs: Default::default(),
                        trs: Default::default(),
                    }));
                temporal_geometry.id.unwrap()
            }
            Some(TemporalGeometry::Complex(ref mut tg)) => {
                // This re-uses ids, might lead to surprising behaviour when deleting a tg and then
                // adding a new one
                temporal_geometry.id = Some(tg.prisms.len().to_string());
                tg.prisms.push(temporal_geometry.clone());
                temporal_geometry.id.unwrap()
            }
        };
        self.update_feature(&feature).await?;
        Ok(id)
    }
    async fn read_temporal_geometry(
        &self,
        collection: &str,
        m_feature_id: &str,
    ) -> anyhow::Result<Option<TemporalGeometry>> {
        let feature = self
            .read_feature(collection, m_feature_id, &Default::default())
            .await?
            .ok_or(anyhow!("Feature not found!"))?;
        Ok(feature.temporal_geometry)
    }
    async fn delete_temporal_geometry(
        &self,
        collection: &str,
        m_feature_id: &str,
        t_geometry_id: &str,
    ) -> anyhow::Result<()> {
        let mut feature = self
            .read_feature(collection, m_feature_id, &Default::default())
            .await?
            .ok_or(anyhow!("Feature not found!"))?;
        match feature.temporal_geometry {
            Some(TemporalGeometry::Primitive(tg)) if tg.id.as_ref().is_some_and(|id| id == t_geometry_id) => {
                feature.temporal_geometry = None;
                Ok(())
            }
            Some(TemporalGeometry::Complex(ref mut tg)) => {
                if tg.prisms.len() > 2 {
                    tg.prisms
                        .pop_if(|tg| tg.id.as_ref().is_some_and(|tg_id| tg_id == t_geometry_id))
                        .ok_or(anyhow!("Temporal Geometry not found."))?;
                    Ok(())
                } else {
                    Err(anyhow!("Prisms must have at least one value. Try to delete the whole prism."))
                }
            }
            _ => Err(anyhow!(format!("TemporalGeometry with id {t_geometry_id} not found"))),
        }
    }
}
